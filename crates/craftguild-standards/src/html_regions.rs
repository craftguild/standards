//! Inspect embedded executable script syntax before measuring mixed HTML/SVG.

use craftguild_core::Result;
use html5ever::tokenizer::{
    states::RawKind, BufferQueue, CharacterTokens, EndTag, StartTag, TagToken, Token, TokenSink,
    TokenSinkResult, Tokenizer, TokenizerOpts,
};
use std::cell::{Cell, RefCell};
use std::path::Path;

#[derive(Default)]
struct Scripts {
    values: RefCell<Vec<String>>,
    active: Cell<Option<usize>>,
    unsupported: Cell<bool>,
}

impl TokenSink for Scripts {
    type Handle = ();
    fn process_token(&self, token: Token, _: u64) -> TokenSinkResult<()> {
        match token {
            TagToken(tag) if tag.kind == StartTag && &*tag.name == "script" => {
                let kind = tag
                    .attrs
                    .iter()
                    .find(|a| &*a.name.local == "type")
                    .map(|a| a.value.as_ref())
                    .unwrap_or("");
                if matches!(
                    kind,
                    "" | "module" | "text/javascript" | "application/javascript"
                ) {
                    let mut values = self.values.borrow_mut();
                    self.active.set(Some(values.len()));
                    values.push(String::new());
                } else if !matches!(
                    kind,
                    "application/json" | "application/ld+json" | "importmap"
                ) {
                    self.unsupported.set(true);
                }
                return TokenSinkResult::RawData(RawKind::ScriptData);
            }
            TagToken(tag) if tag.kind == EndTag && &*tag.name == "script" => self.active.set(None),
            TagToken(tag)
                if tag.kind == StartTag
                    && matches!(
                        &*tag.name,
                        "style" | "xmp" | "iframe" | "noembed" | "noframes"
                    ) =>
            {
                return TokenSinkResult::RawData(RawKind::Rawtext);
            }
            TagToken(tag) if tag.kind == StartTag && matches!(&*tag.name, "title" | "textarea") => {
                return TokenSinkResult::RawData(RawKind::Rcdata);
            }
            CharacterTokens(text) => {
                if let Some(index) = self.active.get() {
                    self.values.borrow_mut()[index].push_str(&text);
                }
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

pub fn verify(root: &Path, source: &Path) -> Result<()> {
    let input = BufferQueue::default();
    input.push_back(std::fs::read_to_string(source)?.into());
    let tokenizer = Tokenizer::new(Scripts::default(), TokenizerOpts::default());
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    if !input.is_empty()
        || tokenizer.sink.active.get().is_some()
        || tokenizer.sink.unsupported.get()
    {
        return Err("Cannot reliably classify embedded HTML script syntax".into());
    }
    let temp = tempfile::tempdir()?;
    for (index, text) in tokenizer.sink.values.borrow().iter().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        let path = temp.path().join(format!("inline-{index}.js"));
        std::fs::write(&path, text)?;
        crate::npm_regions::verify_with_resolution(
            root,
            &path,
            source.parent().ok_or("Missing script source parent")?,
        )?;
    }
    Ok(())
}
