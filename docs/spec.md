# `imeoff` 仕様書

## 1. 概要・目的

Windows 11 環境において、現在アクティブ（フォーカス中）のウィンドウの IME 状態を単発実行（ワンショット）で制御するための CLI ツール `imeoff.exe` の仕様を定義する。

主用途は WSL2 上の Neovim（および VSCode の `vscode-neovim` 拡張機能）でインサートモードを抜けた際（`InsertLeave`）に、外部から安全かつ高速に IME を OFF（半角英数）へ戻すことである。

### 設計方針・非ゴール

* **設計方針:**
* バックグラウンド常駐やグローバルキーボードフック（`SetWindowsHookEx`）を行わない。
* 他ウィンドウへのキー入力注入（`SendInput`）を行わない。
* 短時間で起動・処理・終了する単発 CLI とし、EDR / セキュリティソフトの誤検知・ブロックリスクを最小化する。

* **非ゴール（対象外）:**
* OS全体のキーリマップや無変換キー単体での ESC 送信（VSCode/IME 仕様およびセキュリティ方針に照らし、`imeoff.exe` の責務外とする）。
* IME 辞書の編集、候補選択、入力文字の変換制御。

---

## 2. 動作環境・前提条件

* **OS:** Windows 11（64-bit）
* **呼び出し元:**
* WSL2（Ubuntu 等）上の Neovim（`/mnt/c/.../imeoff.exe` 経由での非同期呼び出し）
* VSCode（`vscode-neovim` 連携時の Neovim バックエンド）
* Windows コマンドプロンプト / PowerShell 等からの直接実行

* **ビルド環境（Rust 例）:** `x86_64-pc-windows-msvc` または `x86_64-pc-windows-gnu`

---

## 3. CLI インターフェース仕様

### 3.1 コマンド構文

```text
imeoff.exe [0|1]

```

### 3.2 引数と挙動

| 引数 | 動作 | 送信する IME 状態（`IMC_SETOPENSTATUS`） |
| --- | --- | --- |
| なし | IME を OFF にする（デフォルト） | `0`（非アクティブ / 半角英数） |
| `0` | IME を OFF にする | `0`（非アクティブ / 半角英数） |
| `1` | IME を ON にする | `1`（アクティブ / 日本語入力） |
| 上記以外 | 警告なしで `0`（OFF）として扱う、または仕様に合わせて既定動作へフォールバック | `0` |

### 3.3 終了コード（Exit Code）

| 終了コード | 意味 |
| --- | --- |
| `0` | 対象ウィンドウを特定し、IME 制御メッセージ（`WM_IME_CONTROL`）の送信を完了 |
| `1` | フォアグラウンドウィンドウまたはデフォルト IME ウィンドウが取得できない等の失敗 |

---

## 4. 内部処理仕様（Win32 API）

1. **フォアグラウンドウィンドウ取得:**

* `GetForegroundWindow()` を呼び出し、現在アクティブなトップレベルウィンドウハンドル（`HWND`）を取得する。Null の場合は終了コード `1` で終了する。

2. **フォーカス子ウィンドウの解決:**

* Electron（VSCode）やターミナル系アプリではトップレベル `HWND` ではなく内部フォーカスウィンドウ側に IME コンテキストが紐づく場合があるため、`GetWindowThreadProcessId()` でスレッド ID を取得し、`GetGUIThreadInfo()` の `hwndFocus` を優先的にターゲットとする。`hwndFocus` が Null の場合はフォアグラウンド `HWND` にフォールバックする。

3. **IME ウィンドウ取得と状態変更:**

* `ImmGetDefaultIMEWnd(target_hwnd)` でデフォルト IME ウィンドウ（`HWND`）を取得する。Null の場合は終了コード `1` で終了する。
* `SendMessageW(ime_wnd, WM_IME_CONTROL, IMC_SETOPENSTATUS, status)`（`WM_IME_CONTROL = 0x0283`、`IMC_SETOPENSTATUS = 0x0006`）を送信する。

4. **即時終了:**

* 残余処理や待機を行わず、終了コードを返してプロセスを終了する。

---

## 5. Neovim 連携仕様（参考）

WSL2 側の Neovim 設定（`init.lua` および `vscode_init.lua`）において、`InsertLeave` イベント発生時に非同期で `imeoff.exe` を呼び出す。

```lua
vim.api.nvim_create_autocmd("InsertLeave", {
  callback = function()
    vim.fn.jobstart({ "/mnt/c/tools/imeoff.exe" }, { detach = true })
  end,
})

```

* **非同期実行（`detach = true`）の理由:** ノーマルモード復帰時の体感遅延を防ぎ、外部プロセス起動待ちでエディタ操作をブロックしないため。

---

## 6. 受け入れ基準（テスト観点）

* **単体テスト（IME OFF）:** 日本語入力（あ）状態で `imeoff.exe`（引数なし）および `imeoff.exe 0` を実行し、IME が半角英数（A）に切り替わり、終了コードが `0` になること。
* **単体テスト（IME ON）:** 半角英数状態で `imeoff.exe 1` を実行し、IME が日本語入力状態に切り替わり、終了コードが `0` になること。
* **Neovim / `vscode-neovim` 連携テスト:** インサートモードで日本語入力後にノーマルモードへ戻った際、入力遅延や文字欠損を起こさずに IME が OFF になること。
* **セキュリティ・安定性確認:** 常駐プロセスが残らないこと、および利用環境（Windows Defender / EDR）で不審なフック・キー注入として扱われないこと。
