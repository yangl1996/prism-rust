package main

import (
	"os"
	"io/ioutil"
	"encoding/json"
)

type Topology struct {
	BtcdConnections []BtcdConnection `json:"btcd_connections"`
	Nodes []Node `json:"nodes"`
	Miner string `json:"miner"`
	LndChannels []LndChannel `json:"lnd_channels"`
	Demands []Demand `json:"demands"`
}

type BtcdConnection struct {
	Source string `json:"src"`
	Destination string `json:"dst"`
}

type Node struct {
	Name string `json:"name"`
	IP string `json:"ip"`
}

type LndChannel struct {
	Source string `json:"src"`
	Destination string `json:"dst"`
}

type Demand struct {
	Source string `json:"src"`
	Destination string `json:"dst"`
	Rate int `json:"rate"`
}

func parseTopo (filename string) *Topology {
	jsonFile, _ := os.Open(filename)
	defer jsonFile.Close()

	bytes, _ := ioutil.ReadAll(jsonFile)

	var result Topology
	json.Unmarshal(bytes, &result)

	return &result
}
