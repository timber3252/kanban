'use strict';

const fs = require('fs');
const express = require('express');
const bodyParser = require('body-parser');
const WebSocketServer = require('ws').Server;

const { decode, resolve404 } = require('./utils');
const { exit } = require('process');

const kanban = express();
kanban.use(bodyParser.json());         // to support JSON-encoded bodies
kanban.use(bodyParser.urlencoded({     // to support URL-encoded bodies
  extended: true
}));

// Static site
kanban.use(express.static('public'));
kanban.use(resolve404(fs.readFileSync('public/404.html')));

kanban.on('request', function(res, req) {

});

module.exports = kanban;

