# imeoff

Windows 11 で、現在アクティブなウィンドウの IME 状態を単発実行で切り替える CLI ツールです。

主用途は、WSL2 上の Neovim（および VSCode の `vscode-neovim`）でインサートモードを抜けたときに、IME を OFF（半角英数）へ戻すことです。

## 特徴

- 常駐しません。グローバルキーボードフック（`SetWindowsHookEx`）も使いません。
- 他ウィンドウへのキー入力注入（`SendInput`）をしません。
- 起動、処理、終了を短時間で終える単発実行です。

## 使い方

```text
imeoff.exe [0|1]
```

| 引数 | 動作 |
| --- | --- |
| なし | IME を OFF にします |
| `0` | IME を OFF にします |
| `1` | IME を ON にします |
| 上記以外 | OFF として扱います |

### 終了コード

| コード | 意味 |
| --- | --- |
| `0` | IME 制御メッセージの送信を完了 |
| `1` | フォアグラウンドウィンドウまたは IME ウィンドウを取得できなかった |

## ビルド

```sh
# GNU ツールチェーン
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# MSVC ツールチェーン（Windows 上）
cargo build --release --target x86_64-pc-windows-msvc
```

成果物は `target/<target>/release/imeoff.exe` に出力されます。

## 仕組み

1. `GetForegroundWindow()` でアクティブウィンドウを取得します。
2. `GetGUIThreadInfo()` の `hwndFocus` があればそれを対象にします。なければアクティブウィンドウを対象にします。
3. `ImmGetDefaultIMEWnd()` で IME ウィンドウを取得し、`WM_IME_CONTROL` / `IMC_SETOPENSTATUS` を送ります。

## Neovim 連携

```lua
vim.api.nvim_create_autocmd("InsertLeave", {
  callback = function()
    vim.fn.jobstart({ "/mnt/c/tools/imeoff.exe" }, { detach = true })
  end,
})
```

`detach = true` により、ノーマルモードへ戻るときに外部プロセスの起動を待ちません。

## 対象外

- OS 全体のキーリマップ、無変換キー単体での ESC 送信
- IME 辞書の編集、候補選択、変換制御
