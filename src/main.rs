use std::env;
use std::path::Path;
use std::process;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    arg_check(&args);
    println!("引数チェックが通過しました。メイン処理を開始します。");

    match check_folder_recursively(Path::new(&args[2]), args[3].parse::<i32>().unwrap()) {
        Ok(_) => println!("メイン処理が正常に終了しました。"),
        Err(e) => eprintln!("エラーが発生しました: {}", e),
    }
    println!("メイン処理が終了しました。");
}

fn arg_check(args: &[String]) {

    
    println!("受け取った引数: {:?}", args);
    
    // 引数の数をチェック
    if args.len() != 4 {
        println!("エラー: 引数の数が不正です。");
        println!("使用方法: {} <フォルダパス> <数値>", args[0]);
        process::exit(1);
    }


    // 3番目の引数が数値かどうかをチェック
    if let Err(_) = args[3].parse::<i32>() {
        println!("エラー: 3番目の引数は数値である必要があります。");
        process::exit(1);
    }
        
    // フォルダパスの存在チェック
    if !Path::new(&args[2]).is_dir() {
        println!("エラー: 指定されたフォルダパスが存在しません。");
        process::exit(1);
    }
}

// パスとファイルサイズを受け取り、フォルダを再帰的にチェックする
fn check_folder_recursively(path: &Path, size: i32) -> std::io::Result<()> {

    if path.is_dir() {

        for entry in fs::read_dir(path)? {
            let entry = entry?;            
            let path = entry.path();
            
            if path.is_dir() {
                check_folder_recursively(&path, size)?;
            } else {
                // 受け取ったファイルサイズと比較して、ファイル名を表示する
                if path.metadata()?.len() >= size as u64 {
                    println!("ファイル: {}", path.display());
                }
            }
        }
    }
    
    Ok(())
}