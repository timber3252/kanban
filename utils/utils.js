'use strict';

// decode base64
const atob = (str) => {
  return Buffer.from(str + '='.repeat((3 - str.length % 3) % 3), 'base64').toString();
}

const decode = (key) => (req, res, next) => {
  if ('string' === typeof req.params[key]) {
    req.params[key] = atob(req.params[key]);
  }
  if ('string' === typeof req.query[key]) {
    req.query[key] = req.query[key].split(',').map(atob);
  }
  next();
}

const cors = (req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT');
  res.header('Access-Control-Allow-Headers', 'Content-Type');
  next();
}
const resolve404 = (message) => (req, res, next) => {
  res.status(404).end(message);
}

module.exports = {
  decode,
  cors,
  resolve404,
}
