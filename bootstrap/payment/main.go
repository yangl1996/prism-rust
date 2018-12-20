package main

import (
	"fmt"
	"golang.org/x/net/context"
	"os"
	"time"
	"sync"
	"strconv"
)

func main() {
	etcd := getEtcdKeyClient()

	nodename, _ := os.LookupEnv("NODENAME")
	//nodeip, _ := os.LookupEnv("NODEIP")
	topo := parseTopo("default_topo.json")

	var senderwg sync.WaitGroup
	var recverwg sync.WaitGroup
	for _, demand := range topo.Demands {
		if demand.Source == nodename {
			senderwg.Add(1)
			go func(demand Demand) {
				defer senderwg.Done()
				etcdPath := fmt.Sprintf("/payments/%v/%v/invoice", demand.Source, demand.Destination)
				etcdTotalPath := fmt.Sprintf("/payments/%v/%v/total", demand.Source, demand.Destination)
				etcdSuccPath := fmt.Sprintf("/payments/%v/%v/success", demand.Source, demand.Destination)
				etcdwatch := etcd.Watcher(etcdPath, nil)

				var totMux sync.Mutex
				var succMux sync.Mutex
				numTot := 0
				numSucc := 0

				for {
					resp, _ := etcdwatch.Next(context.Background())
					pr := resp.Node.Value
					go func (pr string) {
						lnd, cleanUp := getLNDClient()
						defer cleanUp()
						payresp, err := sendPayment(lnd, pr)
						totMux.Lock()
						numTot += 1
						etcd.Set(context.Background(), etcdTotalPath, strconv.Itoa(numTot), nil)
						totMux.Unlock()
						if err == nil && payresp.PaymentError == "" {
							succMux.Lock()
							numSucc += 1
							etcd.Set(context.Background(), etcdSuccPath, strconv.Itoa(numSucc), nil)
							succMux.Unlock()
						}
					} (pr)
				}
			} (demand)
		} else if demand.Destination == nodename {
			recverwg.Add(1)
			go func(demand Demand) {
				defer recverwg.Done()
				lnd, cleanUp := getLNDClient()
				defer cleanUp()

				interval := time.Duration(1000000.0 / demand.Rate) * time.Microsecond
				paymentTick := time.Tick(interval)
				for range paymentTick {
					pr, _ := addInvoice(lnd, 100000)
					etcdPath := fmt.Sprintf("/payments/%v/%v/invoice", demand.Source, demand.Destination)
					etcd.Set(context.Background(), etcdPath, pr, nil)
				}
			} (demand)
		}
	}
	recverwg.Wait()
	senderwg.Wait()

}

