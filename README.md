# Folder Check

Folder Check は、指定されたディレクトリ内のファイルサイズをチェックし、特定のサイズ以上のファイルを表示するコマンドラインツール
特定のファイルやディレクトリを無視することも可能
今後はS3に対してもできるように追加開発するかも、あるいは引数を良い感じにするとか

## 機能
- 指定されたディレクトリを再帰的に探索
- 指定されたサイズ以上のファイルを表示
- `.foldercheckignore` ファイルを使用して、特定のファイルやディレクトリを無視

## 使用方法
cargo run <ディレクトリパス> <サイズ（バイト）>

## 開発用
### テスト実行
cargo test --package folder_check --bin folder_check -- tests --nocapture