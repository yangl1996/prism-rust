/* global _ */

/*
 * Complex scripted dashboard
 * This script generates a dashboard object that Grafana can load. It also takes a number of user
 * supplied URL parameters (in the ARGS variable)
 *
 * Return a dashboard object, or a function
 *
 * For async scripts, return a function, this function must take a single callback function as argument,
 * call this callback function with the dashboard object (look at scripted_async.js for an example)
 */

'use strict';

// accessible variables in this scope
var window, document, ARGS, $, jQuery, moment, kbn;

// Setup some variables
var dashboard;

// All url parameters are available via the ARGS object
var ARGS;

// Initialize a skeleton with nothing but a rows array and service object
dashboard = {
	panels: [],
};

// Set a title
dashboard.title = 'Prism';

// Set default time
// time can be overridden in the url using from/to parameters, but this is
// handled automatically in grafana core during dashboard initialization
dashboard.time = {
	from: "now-5m",
	to: "now"
};

dashboard.refresh = '1s';

dashboard.timepicker = {
	"refresh_intervals": [
		"1s",
		"3s",
		"5s",
		"10s",
		"30s",
		"1m"
	],
	"time_options": [
		"1m",
		"5m",
		"10m",
		"30m",
		"1h"
	]
};

var nodes = 1;
if(!_.isUndefined(ARGS.nodes)) {
	nodes = parseInt(ARGS.nodes, 10);
}

// prepare the per-node transaction rate queries
var txRateTargets;
txRateTargets = [];
for (var i = 0; i < nodes; i++) {
	txRateTargets.push({
		"target": "node_" + i.toString() + ":confirmed_tx",
		"type": "timeserie"
	});
}

// transaction rate row
dashboard.panels.push({
	"cacheTimeout": null,
	"format": "short",
	"postfix": "Tx/s",
	"postfixFontSize": "50%",
	"targets": [
		{
			"refId": "A",
			"target": "average:confirmed_tx",
			"type": "timeserie"
		}
	],
	"title": "Current Confirmation Rate",
	gridPos: {
		"h": 8,
		"w": 4,
		"x": 0,
		"y": 2
	},
	"type": "singlestat",
	"valueFontSize": "80%",
	"valueName": "current",
	"sparkline": {
		"fillColor": "rgba(31, 118, 189, 0.18)",
		"full": false,
		"lineColor": "rgb(31, 120, 193)",
		"show": true
	}
});

dashboard.panels.push({
	title: 'Transaction Count',
	gridPos: {
		"h": 10,
		"w": 8,
		"x": 4,
		"y": 0
	},
	type: 'graph',
	fill: 1,
	linewidth: 2,
	targets: [
		{
			"refId": "A",
			"target": "accumulative:confirmed_tx",
			"type": "timeserie"
		},
		{
			"refId": "B",
			"target": "accumulative:generated_tx",
			"type": "timeserie"
		}
	],
	tooltip: {
		shared: true
	},
	legend: {
		"avg": true,
		"current": false,
		"max": false,
		"min": false,
		"show": true,
		"total": false,
		"values": true
	}
});
dashboard.panels.push({
	title: 'Per-node Confirmation Rate',
	gridPos: {
		"h": 10,
		"w": 12,
		"x": 12,
		"y": 0
	},
	type: 'graph',
	fill: 0,
	linewidth: 1,
	targets: txRateTargets,
	tooltip: {
		shared: true
	},
	legend: {
		"avg": false,
		"current": false,
		"max": false,
		"min": false,
		"show": false,
		"total": false,
		"values": false
	},
});
dashboard.panels.push({
	"cacheTimeout": null,
	"format": "short",
	"postfix": "Tx/s",
	"postfixFontSize": "50%",
	"targets": [
		{
			"refId": "A",
			"target": "average:confirmed_tx",
			"type": "timeserie"
		}
	],
	"title": "Average Confirmation Rate",
	gridPos: {
		"h": 2,
		"w": 4,
		"x": 0,
		"y": 0
	},
	"type": "singlestat",
	"valueFontSize": "80%",
	"valueName": "avg",
});

return dashboard;
