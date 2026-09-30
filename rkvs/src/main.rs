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

    match stream.write(b"OK\n"){
        Ok(_)=>{},
        Err(e)=>eprintln!("write error: {}",e)
    }

    match kvs.set("hello", "world"){
        Some(s)=> println!("map had same key and value was updated./n before key is {}",s),
        None=> println!("map didn't have same key"),
    };
    match kvs.get("hello"){
        Some(s)=> println!("value is {}",s),
        None=> println!("key,value was not found.")
    }
    kvs.del("hello");
    match kvs.get("hello"){
        Some(s)=> println!("key was deleted.value was {}",s),
        None => println!("key was None when try deleting"),
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
