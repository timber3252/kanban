#!/usr/bin/node

const fs = require('fs');
const https = require('https');

const express = require('express');
const vhost = require('vhost');

const options = {
  cert: fs.readFileSync('cert.crt'),
  key: fs.readFileSync('cert.key'),
}

const kanban = require('./utils');

if (process.argv[2] === 'local') {
  https.createServer(options, kanban).listen(20000);
  console.log('kanban    --> https://localhost:20000');
} else {
  const app = express();
  app.use(vhost('kanban.timber3252.me', kanban));
  https.createServer(options, app).listen(443);

  // redirect http to https
  const http = express();
  http.get('*', (req, res) => {
    if (req.get('User-Agent') === 'Go-http-client/1.1') {
      res.status(403).end('nmsl');
    } else {
      res.redirect('https://' + req.headers.host + req.url);
    }
  });
  http.listen(80);
  console.log('as always have a nice day!');
}
