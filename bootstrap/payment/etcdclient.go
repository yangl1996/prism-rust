package main

import (
	etcdclient "go.etcd.io/etcd/client"
	"time"
	"log"
)

func getEtcdKeyClient() etcdclient.KeysAPI {
	etcdcfg := etcdclient.Config{
		Endpoints:               []string{"http://127.0.0.1:2379"},
		Transport:               etcdclient.DefaultTransport,
		HeaderTimeoutPerRequest: time.Second,
	}
	etcdc, err := etcdclient.New(etcdcfg)
	if err != nil {
		log.Fatal(err)
	}
	kapi := etcdclient.NewKeysAPI(etcdc)
	return kapi
}

