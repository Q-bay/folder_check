use std::env;
use std::path::Path;
use std::process;

fn main() {
    arg_check();
    // 引数チェックが通過した場合の処理をここに記述
    println!("引数チェックが通過しました。メイン処理を開始します。");
}

fn arg_check() {
    let args: Vec<String> = env::args().collect();
    
    println!("受け取った引数: {:?}", args);
    
    // 引数の数をチェック
    if args.len() != 3 {
        println!("エラー: 引数の数が不正です。");
        println!("使用方法: {} <フォルダパス> <数値>", args[0]);
        process::exit(1);
    }


    // 2番目の引数が数値かどうかをチェック
    if let Err(_) = args[2].parse::<i32>() {
        println!("エラー: 2番目の引数は数値である必要があります。");
        process::exit(1);
    }
        
    // フォルダパスの存在チェック
    if !Path::new(&args[1]).is_dir() {
        println!("エラー: 指定されたフォルダパスが存在しません。");
        process::exit(1);
    }
}