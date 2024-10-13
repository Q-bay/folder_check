use std::process;
use std::path::Path;
use std::fs;
use std::io::{self, BufRead};
use regex::Regex;
use std::error::Error;
use aws_sdk_s3::Client as S3Client;
use aws_config;
use clap::Parser;
use aws_sdk_s3::types::Object;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Source type: 's3' or 'local'
    #[clap(short = 'r', long, default_value = "local")]
    source: String,

    /// Path to local directory or S3 bucket name
    #[clap(short, long)]
    path: String,

    /// S3 prefix (only for S3 source)
    #[clap(short = 'x', long)]
    prefix: Option<String>,

    /// Minimum file size in bytes
    #[clap(short, long)]
    size: u64,
}

// 無視するパターンを保持する構造体
#[derive(Debug)]
struct IgnorePatterns {
    paths: Vec<Regex>,
    extensions: Vec<String>,
}

const IGNORE_FILE_PATH: &str = ".foldercheckignore";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    
    if let Err(e) = arg_check(&args) {
        eprintln!("エラー: {}", e);
        process::exit(1);
    }
    
    println!("引数チェックが通過しました。メイン処理を開始します。");

    // .foldercheckignoreファイルから無視パターンを読み込む
    let ignore_patterns = load_ignore_patterns(IGNORE_FILE_PATH).unwrap_or_else(|e| {
        // エラー時
        eprintln!("警告: 無視パターンの読み込みに失敗しました: {}", e);
        IgnorePatterns { paths: vec![], extensions: vec![] }
    });
    println!("{:?}", ignore_patterns);

    match args.source.as_str() {
        "s3" => check_s3(&args, &ignore_patterns).await?,
        "local" => check_local(&args, &ignore_patterns)?,
        _ => return Err("Invalid source type. Use 's3' or 'local'.".into()),
    }

    println!("処理が正常に終了しました。");
    Ok(())
}



fn arg_check(args: &Args) -> Result<(), Box<dyn Error>> {
    println!("受け取った引数: {:?}", args);

    if args.size == 0 {
        return Err("サイズは0より大きい必要があります。".into());
    }
    
    if args.source == "local" && !Path::new(&args.path).is_dir() {
        return Err("指定されたフォルダパスが存在しません。".into());
    }

    Ok(())
}

// 無視パターンをファイルから読み込む関数
fn load_ignore_patterns(filename: &str) -> io::Result<IgnorePatterns> {

    let file = fs::File::open(filename)?;

    let reader = io::BufReader::new(file);

    // 空のベクターを空のパスと拡張子のベクターとして初期化 
    let mut paths = Vec::new();
    let mut extensions = Vec::new();

    for line in reader.lines() {
        let line = line?;

        let trimmed_line = line.trim();

        // コメント行または空行はスキップ   
        if trimmed_line.starts_with('#') || trimmed_line.is_empty() {
            continue;
        }

        // '*'で始まる行は拡張子パターン
        if trimmed_line.starts_with('*') {
            extensions.push(trimmed_line[1..].to_string());
        } else {
            // パスパターン 
            paths.push(Regex::new(&format!("(^|/){}/", regex::escape(trimmed_line))).unwrap());
        }
    }

    Ok(IgnorePatterns { paths, extensions })
}

fn check_local(args: &Args, ignore: &IgnorePatterns) -> Result<(), Box<dyn Error>> {
    check_folder_recursively(Path::new(&args.path), args.size, ignore).map_err(|e| e.into())
}
// フォルダを再帰的にチェックする関数
fn check_folder_recursively(path: &Path, size: u64, ignore: &IgnorePatterns) -> io::Result<()> {
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;            
        let path = entry.path();
        
        if should_ignore(&path, ignore) {
            continue;
        }
        
        if path.is_dir() {
            check_folder_recursively(&path, size, ignore)?;
        } else {
            // ファイルサイズをチェックし、指定サイズ以上なら表示
            if path.metadata()?.len() >= size as u64 {
                println!("ファイル: {}", path.display());
            }
        }
    }
    
    Ok(())
}

// ファイルやディレクトリを無視すべきかどうかを判断する関数
fn should_ignore(path: &Path, ignore: &IgnorePatterns) -> bool {
    let path_str = path.to_str().unwrap_or("").replace('\\', "/");

    // ignore.pathsの要素に一致した場合無視する
    if ignore.paths.iter().any(|re| re.is_match(&path_str)) {
        return true;
    }

    // ファイルの拡張子を取得して、無視リストに含まれているかチェック   
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_str().unwrap_or("").to_lowercase();
        if ignore.extensions.iter().any(|ignored_ext| ignored_ext.trim_start_matches('.').to_lowercase() == ext_str) {
            return true;
        }
    }

    false
}

async fn check_s3(args: &Args, ignore: &IgnorePatterns) -> Result<(), Box<dyn Error>> {
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
    let client = S3Client::new(&config);

    let result = client.list_objects_v2()
        .bucket(&args.path)
        .prefix(args.prefix.as_deref().unwrap_or(""))
        .send()
        .await?;

    if let Some(contents) = result.contents {
        for object in contents {
            process_s3_object(&object, args, ignore);
        }
    }

    Ok(())
}

fn process_s3_object(object: &Object, args: &Args, ignore: &IgnorePatterns) {
    let key = match &object.key {
        Some(k) => k,
        None => return, // キーがない場合は早期リターン
    };

    if should_ignore(Path::new(key), ignore) {
        return; // 無視すべきオブジェクトの場合は早期リターン
    }

    let size = object.size.unwrap_or_default() as u64;
    if size < args.size {
        return; // サイズが指定値未満の場合は早期リターン
    }

    // プレフィックスを含めた形式で出力
    let formatted_key = format!("/{}", key.trim_start_matches('/'));
    println!("S3 Object: {}, Size: {} bytes", formatted_key, size);
}

#[cfg(test)]
mod tests;