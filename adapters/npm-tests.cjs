"use strict";

const fs = require("node:fs");
const path = require("node:path");

/** Reject mixed test source when reliable region ownership is unavailable. */
function classify(source, filename, ts) {
  const kind = filename.endsWith(".tsx") ? ts.ScriptKind.TSX
    : filename.endsWith(".jsx") ? ts.ScriptKind.JSX
    : /\.[cm]?ts$/.test(filename) ? ts.ScriptKind.TS : ts.ScriptKind.JS;
  const syntax = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true, kind);
  if (syntax.parseDiagnostics.length) {
    throw new Error("Invalid JavaScript/TypeScript syntax");
  }
  const names = new Set(["test", "it", "describe", "suite"]);
  const testModules = new Set(["node:test", "vitest", "@jest/globals", "mocha"]);
  for (const statement of syntax.statements) {
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        let call = declaration.initializer;
        while (call && ts.isPropertyAccessExpression(call)) call = call.expression;
        if (!call || !ts.isCallExpression(call) || !ts.isIdentifier(call.expression)
          || call.expression.text !== "require" || call.arguments.length !== 1
          || !ts.isStringLiteral(call.arguments[0]) || !testModules.has(call.arguments[0].text)) continue;
        if (ts.isIdentifier(declaration.name)) names.add(declaration.name.text);
        if (ts.isObjectBindingPattern(declaration.name)) {
          for (const binding of declaration.name.elements) {
            if (ts.isIdentifier(binding.name)) names.add(binding.name.text);
          }
        }
      }
    }
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) continue;
    const moduleName = statement.moduleSpecifier.text;
    if (!testModules.has(moduleName)) continue;
    const clause = statement.importClause;
    if (clause?.name) names.add(clause.name.text);
    const bindings = clause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings)) names.add(bindings.name.text);
    if (bindings && ts.isNamedImports(bindings)) {
      for (const binding of bindings.elements) names.add(binding.name.text);
    }
  }
  let mixed = false;
  function visit(node) {
    if (ts.isCallExpression(node)) {
      let callee = node.expression;
      while (ts.isPropertyAccessExpression(callee)) callee = callee.expression;
      if (ts.isIdentifier(callee) && names.has(callee.text)) mixed = true;
    }
    ts.forEachChild(node, visit);
  }
  visit(syntax);
  return { mixed };
}

module.exports = { classify };

if (require.main === module) {
  try {
    const [filename, root] = process.argv.slice(2);
    if (!filename || !root) throw new Error("Expected source and repository paths");
    const tsPath = require.resolve("typescript", { paths: [path.dirname(filename), root] });
    const ts = require(tsPath);
    process.stdout.write(JSON.stringify(classify(fs.readFileSync(filename, "utf8"), filename, ts)));
  } catch (error) {
    console.error(String(error));
    process.exitCode = 1;
  }
}
