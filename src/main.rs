use std::env;

fn main() {
    arg_check();
    println!("Hello, world!");
}

fn arg_check() {
    let args: Vec<String> = env::args().collect();
    
    // 引数の数をチェック
    if args.len() >= 4 {
        println!("使用方法: {} <フォルダパス> <数値>", args[0]);
        return;
    }

    println!("{:?}", args);
}