use std::sync::Mutex;
use std::collections::HashMap;
mod kvs;
use kvs::Kvs;


use std::net::{TcpListener,TcpStream};
use std::io::Read;
use std::io::Write;

fn handle_client(mut stream:TcpStream,kvs:&Kvs){
    // mut にすることで可変にする。ストリームの読み書きで使うから都度値が更新される
    // 配列にする理由は、スタック領域に乗るから。
    let mut buf = [0u8; 1024];
    // &mut にすることで可変参照にしている。つまり呼び出し側でbufの値を変更できる
    // &buf にすると不変参照 
    let n = match stream.read(&mut buf){
        Ok(n) => n,
        Err(e) =>{
            eprintln!("read error: {}",e);
            return;
        }
    };

    // from_ut8_lossyを使うとバイト列から文字列にutf-8で読み込む
    // このとき異常なバイト列があってもエラーにせず、?で置き換える
    // なにがネットワーク越しに来るかわからないのでfrom_ut8_lossyを使うのが定石
    let received = String::from_utf8_lossy(&buf[..n]);
    println!("{}",received);

    // 受け取った文字列を1行ずつコマンドとして処理し、結果をクライアントに返す
    let mut response = String::new();
    for line in received.lines(){
        if line.trim().is_empty(){
            continue;
        }
        response.push_str(&execute_command(kvs,line));
        response.push('\n');
    }

    if let Err(e) = stream.write_all(response.as_bytes()){
        eprintln!("write error: {}",e);
    }
}

// "GET key" / "SET key value" / "DEL key" をKvsのメソッドに振り分ける
// SETのvalueは空白を含んでもよい(keyの後ろ全部をvalueとする)
fn execute_command(kvs:&Kvs,line:&str)->String{
    let mut parts = line.trim().splitn(3,char::is_whitespace);
    let cmd = parts.next().unwrap_or("").to_uppercase();
    let key = parts.next();
    let value = parts.next().map(|v| v.trim());

    match (cmd.as_str(),key,value){
        ("GET",Some(k),None)=>match kvs.get(k){
            Some(v)=>v,
            None=>"(nil)".to_string(),
        },
        ("SET",Some(k),Some(v)) if !v.is_empty()=>match kvs.set(k,v){
            Some(_)=>"OK (updated)".to_string(),
            None=>"OK".to_string(),
        },
        ("DEL",Some(k),None)=>match kvs.del(k){
            Some(_)=>"OK".to_string(),
            None=>"(nil)".to_string(),
        },
        ("GET",..)|("SET",..)|("DEL",..)=>"ERR wrong number of arguments (GET key | SET key value | DEL key)".to_string(),
        _=>format!("ERR unknown command '{}'",cmd),
    }
}


fn run() -> Result<(),Box<dyn std::error::Error>> {
    
    let kvs: Kvs = Kvs::new();

    // match文の各バリアントへの操作をアームという
    // アームはすべて同じ型を返す必要がある。
    // ただしreturnがあるアームはmatch式の型評価から外れるので型の不一致にならない
    // で、慣習としてエラーの場合はreturnで呼び出し元の関数に対してResult型のErr(e)バリアントを返すのが慣習らしい
    // ?を使うと同じことをmatch無しでかける
    // unwrap()を使うとpanicになって全体終了する。だからエラーハンドリングを書かなくて良くなるが、エラーに対しての対応ができなくなる。実験とか学習向けにはいい。
    let listner = match TcpListener::bind("127.0.0.1:8080"){//linuxでは1024番以下のポートを使うにはroot権限が必要。127.0.0.1:80を権限なしユーザーで実行するとPermissiondeniedでエラーになる
        Ok(l)=> l,
        Err(e)=> return Err(e.into()),
    };

    // listenr.incoming()で無限ループをおこない、クライアントからの接続があったとき、stream:Result<TcpStream>を返す。
    for stream in listner.incoming(){
        match stream{
            Ok(s)=>handle_client(s,&kvs),
            Err(e)=>return Err(e.into())
        }
    }
    Ok(())
}

// 慣習としてはmain文にはロジックを書かずに、ロジックを書いたメソッド呼んでそのメソッドがエラー出したときにプログラム全体をエラーにするらしい。
// std::process::exit(1)でステータスコード1で終了する
// eprintln!で標準エラー出力に出力
fn main() {
    if let Err(e) = run(){
        eprintln!("{}",e);
        std::process::exit(1);
    }
}
