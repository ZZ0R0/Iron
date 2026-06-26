import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";
import { execFile } from "child_process";

let client: LanguageClient;

function ironcPath(): string {
  return vscode.workspace.getConfiguration("iron").get<string>("path", "ironc");
}

export function activate(context: vscode.ExtensionContext) {
  // --- Client LSP : lance `ironc lsp` (stdio) → diagnostics/complétion/hover live (71300/71700) ---
  const serverOptions: ServerOptions = {
    run: { command: ironcPath(), args: ["lsp"] },
    debug: { command: ironcPath(), args: ["lsp"] },
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "iron" }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher("**/*.irn") },
  };
  client = new LanguageClient("iron", "Iron Language Server", serverOptions, clientOptions);
  client.start();

  // --- Diagnostics overlay rustc (`ironc check`) en complément, dans le panneau Problems (60800/71700) ---
  const diag = vscode.languages.createDiagnosticCollection("iron-check");
  context.subscriptions.push(diag);
  const checkOnSave = (doc: vscode.TextDocument) => {
    if (doc.languageId !== "iron") return;
    if (!vscode.workspace.getConfiguration("iron").get<boolean>("checkOnSave", true)) return;
    execFile(ironcPath(), ["check", doc.fileName], (_err, _stdout, stderr) => {
      diag.delete(doc.uri);
      // remontée ironc : "fichier:ligne:col: erreur: message"
      const m = /:(\d+):(\d+):\s*(.*)/.exec(stderr);
      if (m) {
        const line = parseInt(m[1]) - 1, col = parseInt(m[2]) - 1;
        const range = new vscode.Range(line, col, line, col + 1);
        diag.set(doc.uri, [new vscode.Diagnostic(range, m[3], vscode.DiagnosticSeverity.Error)]);
      }
    });
  };
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(checkOnSave));

  // --- Commandes : add (import Rust), build, run, test, format (71800/71900) ---
  const term = () => vscode.window.activeTerminal ?? vscode.window.createTerminal("Iron");
  const file = () => vscode.window.activeTextEditor?.document.fileName ?? "";
  const reg = (id: string, fn: () => void) => context.subscriptions.push(vscode.commands.registerCommand(id, fn));

  reg("iron.addCrate", async () => {
    const crate = await vscode.window.showInputBox({ prompt: "Crate Rust à importer (ex. regex)" });
    if (crate) term().sendText(`${ironcPath()} add ${crate}`);
  });
  reg("iron.build", () => term().sendText(`${ironcPath()} build "${file()}"`));
  reg("iron.run", () => { const t = term(); t.show(); t.sendText(`${ironcPath()} run "${file()}"`); });
  reg("iron.test", () => term().sendText(`${ironcPath()} test "${file()}"`));
  reg("iron.format", () => term().sendText(`${ironcPath()} fmt "${file()}"`));

  // --- CodeLens « ▶ Run » au-dessus de `def main()` (71900) ---
  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider({ language: "iron" }, {
      provideCodeLenses(doc) {
        const lenses: vscode.CodeLens[] = [];
        for (let i = 0; i < doc.lineCount; i++) {
          if (/^\s*def main\s*\(/.test(doc.lineAt(i).text)) {
            lenses.push(new vscode.CodeLens(new vscode.Range(i, 0, i, 0), { title: "▶ Run", command: "iron.run" }));
          }
        }
        return lenses;
      },
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
