'use strict';

function setCookie(name, value, expire_days) {
  let cookie_str = name + "=" + escape(value); 
  if (expire_days != null) {
    cookie_str += ";path=/;max-age=" + expire_days * 24 * 3600;
  }
  document.cookie = cookie_str;
}

function getCookie(name) {
  if (document.cookie.length > 0) {
    let cookies = document.cookie.split(';');
    for (let x of cookies) {
      let key = x.split('=')[0], value = x.split('=')[1];
      if (key == name) return value;
    }
  }
  return "";
}
