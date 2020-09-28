use std::net::TcpListener;
use std::thread::spawn;
use std::result::Result;
use tungstenite::{Message, server::accept};
use postgres::{Client, NoTls};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use digest::Digest;
use chrono::NaiveDateTime;

struct Task {
  id: i32,
  userid: String,
  task_name: String,
  task_ddl: String,
  task_desc: String,
  task_group: String,
  mark: bool
}

fn init() {
  let mut client = match Client::connect("host=localhost user=postgres dbname=kanban", NoTls) {
    Ok(c) => c,
    Err(err) => {
      if err.code().unwrap().code() == "3D000" {
        let mut c = Client::connect("host=localhost user=postgres", NoTls).unwrap();
        c.batch_execute("CREATE DATABASE kanban").unwrap();
        Client::connect("host=localhost user=postgres dbname=kanban", NoTls).unwrap()
      } else {
        panic!("{:?}", err);
      }
    }
  };
  client.batch_execute("
    CREATE TABLE IF NOT EXISTS users (
        id          SERIAL PRIMARY KEY,
        username    TEXT NOT NULL,
        userpwd     TEXT NOT NULL,
        userid      TEXT NOT NULL
    )
  ").unwrap();
  client.batch_execute("
    CREATE TABLE IF NOT EXISTS tasks (
        id          SERIAL PRIMARY KEY,
        userid      TEXT NOT NULL,
        task_name   TEXT NOT NULL,
        task_ddl    TIMESTAMP WITHOUT TIME ZONE,
        task_desc   TEXT NOT NULL,
        task_group  TEXT NOT NULL,
        mark        BOOLEAN
    )
  ").unwrap();
}

fn main() {
  init();
  let server = TcpListener::bind("127.0.0.1:26868").unwrap();
  // Protocol (Recv):
  //    QueryUser#uid (OK)
  //    UpdateLists#uid (OK)
  //    Register#name#pwd (OK)
  //    Login#name#pwd (OK)
  //    NewTask#uid#{context,project,maybe,finish}#task_name#task_time#task_desc (OK)
  //    DelTask#id (OK)
  //    MoveTask#id#TO (OK)
  // Protocol (Send):
  //    QueryUser#{true,false} (OK)
  //    UpdateList#{context,project,maybe,finish}#RAW_HTML_DATA(ESCAPE) (OK)
  //    Login#{true#uid,Password Error} (OK)
  //    Register#{true,false} (OK)
  for stream in server.incoming() {
    spawn(move || {
      let mut client = match Client::connect("host=localhost user=postgres dbname=kanban", NoTls) {
        Ok(c) => c,
        Err(err) => {
          if err.code().unwrap().code() == "3D000" {
            let mut c = Client::connect("host=localhost user=postgres", NoTls).unwrap();
            c.batch_execute("CREATE DATABASE kanban").unwrap();
            Client::connect("host=localhost user=postgres dbname=kanban", NoTls).unwrap()
          } else {
            panic!("{:?}", err);
          }
        }
      };
      let mut ws = accept(stream.unwrap()).unwrap();
      let ws_send = move |data: String, ws: &mut tungstenite::WebSocket<std::net::TcpStream>| {
        match ws.write_message(Message::Text(data)) {
          Ok(_) => (),
          Err(e) => {
            println!("{}", e);
            ()
          }
        }
      };
      loop {
        let msg = match ws.read_message() {
          Ok(e) => e,
          Err(_) => {
            break;
          }
        };
        println!("{}", msg);
        let msg: Vec<&str> = msg.to_text().unwrap().split('#').collect();
        if msg[0] == "QueryUser" {
          let (key, uid) = (String::from(msg[0]), String::from(msg[1]));
          let mut flag: bool = false;
          for _ in client.query("SELECT * FROM users WHERE userid = $1", &[&uid]).unwrap() {
            flag = true;
          }
          ws_send(key + "#" + if flag { "true" } else { "false" }, &mut ws);
        } else if msg[0] == "UpdateLists" {
          let uid = String::from(msg[1]);
          ws_send(String::from("UpdateList#.context-panel#") + &query_tasks(&uid, "context", &mut client), &mut ws);
          ws_send(String::from("UpdateList#.project-panel#") + &query_tasks(&uid, "project", &mut client), &mut ws);
          ws_send(String::from("UpdateList#.maybe-panel#")   + &query_tasks(&uid, "maybe", &mut client), &mut ws);
          ws_send(String::from("UpdateList#.finish-panel#")  + &query_tasks(&uid, "finish", &mut client), &mut ws);
        } else if msg[0] == "Register" {
          let (usr, pwd) = (String::from(msg[1]), String::from(msg[2]));
          let uid = calc_userid(&usr, &pwd);
          let mut exist: bool = false;
          for _ in client.query("SELECT * FROM users WHERE username = $1", &[&usr]).unwrap() {
            exist = true;
          }
          if exist {
            ws_send(String::from("Register#用户已存在"), &mut ws);
          } else {
            match client.execute("
              INSERT INTO users (username, userpwd, userid)
              VALUES ($1, $2, $3)
            ", &[&usr, &pwd, &uid]) {
              Ok(_) => {
                ws_send(String::from("Register#true"), &mut ws);
              },
              Err(_) => {
                ws_send(String::from("Register#false"), &mut ws);
              }
            }
          }
        } else if msg[0] == "Login" {
          let (usr, pwd) = (String::from(msg[1]), String::from(msg[2]));
          let mut done: bool = false;
          for row in client.query("SELECT * FROM users WHERE username = $1", &[&usr]).unwrap() {
            let standard_pwd: String = row.get(2);
            let uid = row.get(3);
            if standard_pwd == pwd {
              ws_send(String::from("Login#true#") + uid, &mut ws);
            } else {
              ws_send(String::from("Login#用户名或密码错误"), &mut ws);
            }
            done = true;
          }
          if !done {
            ws_send(String::from("Login#用户名或密码错误"), &mut ws);
          }
        } else if msg[0] == "NewTask" {
           let task = Task {
            id: -1,
            userid: String::from(msg[1]),
            task_name: String::from(msg[3]),
            task_ddl: String::from(msg[4]),
            task_desc: String::from(msg[5]),
            task_group: String::from(msg[2]),
            mark: if String::from(msg[6]) == "true" { true } else { false }
          };
          add_task(&task, &mut client);
        } else if msg[0] == "DelTask" {
          if let Ok(id) = String::from(msg[1]).parse::<i32>() {
            if let Err(e) = del_task(id, &mut client) {
              println!("{:?}", e);
            }
          }
        } else if msg[0] == "MoveTask" {
          if let Ok(tid) = String::from(msg[1]).parse::<i32>() {
            let gid = String::from(msg[2]);
            if gid != "context" && gid != "maybe" && gid != "project" && gid != "finish" {
              continue;
            }
            if let Err(e) = client.execute("UPDATE tasks SET task_group = $1 WHERE id = $2", &[&gid, &tid]) {
              println!("{:?}", e);
            }
          }
        }
      }
    });
  }
}

fn render_task(task: &Task) -> String {
  format!("
    <div class=\"card {0}\" id=\"{1}\">
      <span class=\"task_name\">{2}</span>
      <label class=\"tcheckbox\" hidden><input type=\"checkbox\" class=\"tcheckbox-body\"><span></span></label>
      <br>
      <span class=\"task_time\">{3}</span><br>
      <span class=\"task_desc\">{4}</span>
    </div>
  ", if task.mark { "mark" } else { "default" }, task.id, task.task_name, task.task_ddl, task.task_desc)
}

fn add_task(task: &Task, client: &mut Client) {
  if let Ok(deadline) = NaiveDateTime::parse_from_str(&task.task_ddl, "%Y-%m-%d %H:%M") {
    if let Err(e) = client.execute("
      INSERT INTO tasks (userid, task_name, task_ddl, task_desc, task_group, mark)
      VALUES ($1, $2, $3, $4, $5, $6)
    ", &[&task.userid, &task.task_name, &deadline, &task.task_desc, &task.task_group, &task.mark]) {
      println!("{}", e);
    }
  }
}

fn del_task(id: i32, client: &mut Client) -> Result<(), postgres::Error> {
  client.batch_execute(&format!("DELETE FROM tasks WHERE id = {}", id))
}

fn query_tasks(uid: &str, gid: &str, client: &mut Client) -> String {
  let mut res = String::new();
  for row in client.query("SELECT * FROM tasks WHERE userid = $1 and task_group = $2", &[&uid, &gid]).unwrap() {
    let task = Task {
      id: row.get(0),
      userid: row.get(1),
      task_name: row.get(2),
      task_ddl: {
        let timestamp = row.get::<usize, NaiveDateTime>(3);
        timestamp.format("%Y-%m-%d %H:%M").to_string()
      },
      task_desc: row.get(4),
      task_group: row.get(5),
      mark: row.get(6)
    };
    res = res + &render_task(&task);
  }
  res
}

fn calc_userid(usr: &str, pwd: &str) -> String {
  type HmacSha256 = Hmac<Sha256>;
  if let Ok(mut mac) = HmacSha256::new_varkey(pwd.as_bytes()) {
    mac.update(usr.as_bytes());
    mac.update(pwd.as_bytes());
    return format!("{:x}", Sha256::digest(&mac.finalize().into_bytes()));
  }
  String::from(pwd) + usr
}
