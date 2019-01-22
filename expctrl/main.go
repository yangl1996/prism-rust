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
    fmt.Println("Experiment starting")
	etcd := getEtcdKeyClient()

	nodename, _ := os.LookupEnv("NODENAME")
	topopath, _ := os.LookupEnv("TOPO_FILE")
    paymentSizeStr, _ := os.LookupEnv("PAYMENT_SIZE")
    paymentSize, _ := strconv.ParseInt(paymentSizeStr, 10, 64)
	//nodeip, _ := os.LookupEnv("NODEIP")
	topo := parseTopo(topopath)

    var printfMux sync.Mutex

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
						totMux.Lock()
						numTot += 1
						etcd.Set(context.Background(), etcdTotalPath, strconv.Itoa(numTot), nil)
						totMux.Unlock()
                        printfMux.Lock()
                        fmt.Printf("payment added\n")
                        printfMux.Unlock()
						payresp, err := sendPayment(lnd, pr)
						if err == nil && payresp.PaymentError == "" {
							succMux.Lock()
							numSucc += 1
							etcd.Set(context.Background(), etcdSuccPath, strconv.Itoa(numSucc), nil)
							succMux.Unlock()
						} else {
                            if err != nil {
                                printfMux.Lock()
                                fmt.Printf("payment rpc error: %v\n", err)
                                printfMux.Unlock()
                            } else {
                                printfMux.Lock()
                                fmt.Printf("payment lnd error: %v\n", payresp.PaymentError)
                                printfMux.Unlock()
                            }
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
					pr, _ := addInvoice(lnd, paymentSize)
					etcdPath := fmt.Sprintf("/payments/%v/%v/invoice", demand.Source, demand.Destination)
					etcd.Set(context.Background(), etcdPath, pr, nil)
				}
			} (demand)
		}
	}
	recverwg.Wait()
	senderwg.Wait()

}

