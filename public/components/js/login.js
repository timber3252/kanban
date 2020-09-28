let result = getCookie("KANBAN_USERID");
if (result != "") {
  location.replace('/');
}

function toggleReg() {
  document.querySelector('.loginpart').hidden = true;
  document.querySelector('.regpart').hidden = false;
}

function toggleLogin() {
  document.querySelector('.regpart').hidden = true;
  document.querySelector('.loginpart').hidden = false;
}

function resolvePassword(usr, pwd) {
  let hmac = CryptoJS.algo.HMAC.create(CryptoJS.algo.SHA256, pwd);
  hmac.update("zjutjh");
  hmac.update(usr);
  hmac.update("zjutjh");
  let hash = hmac.finalize();
  return hash.toString(CryptoJS.enc.Hex);
}

function checkUsername(usr) {
  if (usr == "") {
    alert("请输入用户名！");
    return false;pwd = resolvePassword(usr, pwd);
  }
  var patrn = /^[a-zA-Z0-9]{4,32}$/;
  if (!patrn.exec(usr)) {
    alert("用户名只能由字母和数字构成，并且长度在 4 到 32 之间");
    return false;
  }
  return true;
}

function checkPassword(pwd) {
  if (pwd == "") {
    alert("请输入密码");
    return false;
  }
  if (pwd.length < 6 || pwd.length > 32) {
    alert("密码的长度需要在 6 到 32 之间");
    return false;
  }
  return true;
}

function register() {
  let usr = document.getElementById('edt_username2').value;
  let pwd = document.getElementById('edt_password2').value;
  if (checkUsername(usr) && checkPassword(pwd)) {
    pwd = resolvePassword(usr, pwd);
    var ws = new WebSocket("ws://localhost:26868");
    ws.onopen = function (e) {
      console.log('connected to ws server');
      ws.send('Register#' + usr + '#' + pwd);
    }
    ws.onerror = function (e) {
      console.error('fatal error:', e);
    }
    ws.onmessage = function (e) {
      let arr = e.data.split('#');
      if (arr[0] == 'Register') {
        if (arr[1] == 'true') {
          alert("注册成功，请重新登录");
          toggleLogin();
        } else if (arr[1] == 'false') {
          alert("注册出错，请重试");
        } else {
          alert(arr[1]);
        }
      }
    }
  }
}

function login() {
  let usr = document.getElementById('edt_username1').value;
  let pwd = document.getElementById('edt_password1').value;
  if (checkUsername(usr) && checkPassword(pwd)) {
    pwd = resolvePassword(usr, pwd);
    var ws = new WebSocket("ws://localhost:26868");
    ws.onopen = function (e) {
      console.log('connected to ws server');
      ws.send('Login#' + usr + '#' + pwd);
    }
    ws.onerror = function (e) {
      console.error('fatal error:', e);
    }
    ws.onmessage = function (e) {
      let arr = e.data.split('#');
      if (arr[0] == 'Login') {
        if (arr[1] == 'true') {
          setCookie('KANBAN_USERID', arr[2], 30);
          location.replace('/');
        } else {
          alert(arr[1]);
        }
      }
    }
  }
}

function resolveKey() {
  if (event.keyCode == 13) {
    if (document.querySelector('.loginpart').hidden == true) {
      register();
    } else {
      login();
    }
  }
}