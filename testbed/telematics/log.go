package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	tm "github.com/buger/goterm"
	"github.com/ziutek/rrd"
	"net/http"
	"os"
	"os/exec"
	"path"
	"strings"
	"time"
)

type Snapshot struct {
	Generated_transactions                       int
	Confirmed_transactions                       int
	Deconfirmed_transactions                     int
	Incoming_message_queue                       int
	Mined_proposer_blocks                        int
	Mined_voter_blocks                           int
	Mined_transaction_blocks                     int
	Total_proposer_block_delay                   int
	Total_voter_block_delay                      int
	Total_transaction_block_delay                int
	Received_proposer_blocks                     int
	Received_voter_blocks                        int
	Received_transaction_blocks                  int
	Confirmed_transaction_blocks                 int
	Deconfirmed_transaction_blocks               int
	Total_transaction_block_confirmation_latency int
	Proposer_main_chain_length                   int
	Voter_main_chain_length_sum                  int
	Processed_proposer_blocks                    int
	Processed_voter_blocks                       int
}

type expSnapshot struct {
	time               int
	confirmed_tx       int
	confirmed_tx_blk   int
	processed_voter    int
	processed_proposer int
	voter_len_sum      int
	proposer_len       int
	latency_sum        int
}

type Report struct {
	Node string
	Data Snapshot
}

func log(interval, duration uint, nodesFile, dataDir string, grafana bool) {
	fmt.Println("Parsing node list")
	nodes := make(map[string]string)
	file, err := os.Open(nodesFile)
	if err != nil {
		fmt.Println("Error opening node list:", err)
		os.Exit(1)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		s := strings.Split(scanner.Text(), ",")
		name := s[0]
		ip := s[2]
		port := s[5]
		url := fmt.Sprintf("http://%v:%v/telematics/snapshot", ip, port)
		nodes[name] = url
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}

	fmt.Println("Creating round-robin database")
	err = os.MkdirAll(dataDir, os.ModeDir|os.FileMode(0755))
	if err != nil {
		fmt.Println("Error creating data directory", err)
		os.Exit(1)
	}
	for k, _ := range nodes {
		n := path.Clean(dataDir + "/" + k + ".rrd")
		c := rrd.NewCreator(n, time.Now(), interval)
		c.DS("confirmed_tx", "COUNTER", interval*2, 0, "U")
		c.DS("deconfirmed_tx", "COUNTER", interval*2, 0, "U")
		c.DS("generated_tx", "COUNTER", interval*2, 0, "U")
		c.DS("queue_length", "GAUGE", interval*2, 0, "U")
		c.DS("mined_proposer", "COUNTER", interval*2, 0, "U")
		c.DS("mined_voter", "COUNTER", interval*2, 0, "U")
		c.DS("mined_tx", "COUNTER", interval*2, 0, "U")
		c.DS("proposer_delay_sum", "COUNTER", interval*2, 0, "U")
		c.DS("voter_delay_sum", "COUNTER", interval*2, 0, "U")
		c.DS("tx_delay_sum", "COUNTER", interval*2, 0, "U")
		c.DS("received_proposer", "COUNTER", interval*2, 0, "U")
		c.DS("received_voter", "COUNTER", interval*2, 0, "U")
		c.DS("received_tx", "COUNTER", interval*2, 0, "U")
		c.DS("confirmed_tx_blk", "COUNTER", interval*2, 0, "U")
		c.DS("deconfirmed_tx_blk", "COUNTER", interval*2, 0, "U")
		c.DS("txblk_cfm_sum", "COUNTER", interval*2, 0, "U")
		c.DS("prop_chain_depth", "COUNTER", interval*2, 0, "U")
		c.DS("voter_chains_depth", "COUNTER", interval*2, 0, "U")
		c.DS("processed_proposer", "COUNTER", interval*2, 0, "U")
		c.DS("processed_voter", "COUNTER", interval*2, 0, "U")
		c.DS("proposer_delay_mean", "COMPUTE", "proposer_delay_sum,received_proposer,/")
		c.DS("voter_delay_mean", "COMPUTE", "voter_delay_sum,received_voter,/")
		c.DS("tx_delay_mean", "COMPUTE", "tx_delay_sum,received_tx,/")
		c.DS("txblk_cfm_mean", "COMPUTE", "txblk_cfm_sum,confirmed_tx_blk,/,1000,/")
		c.DS("prop_fork", "COMPUTE", "processed_proposer,prop_chain_depth,-,processed_proposer,/")
		c.DS("voter_fork", "COMPUTE", "processed_voter,voter_chains_depth,-,processed_voter,/")
		c.RRA("LAST", 0, 1, duration/interval)
		err = c.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
	}

	if grafana {
		n := path.Clean(dataDir + "/average.rrd")
		cr := rrd.NewCreator(n, time.Now(), interval)
		cr.DS("confirmed_tx", "COUNTER", interval*2, 0, "U")
		cr.DS("deconfirmed_tx", "COUNTER", interval*2, 0, "U")
		cr.DS("generated_tx", "COUNTER", interval*2, 0, "U")
		cr.DS("queue_length", "GAUGE", interval*2, 0, "U")
		cr.DS("mined_proposer", "COUNTER", interval*2, 0, "U")
		cr.DS("mined_voter", "COUNTER", interval*2, 0, "U")
		cr.DS("mined_tx", "COUNTER", interval*2, 0, "U")
		cr.DS("proposer_delay_sum", "COUNTER", interval*2, 0, "U")
		cr.DS("voter_delay_sum", "COUNTER", interval*2, 0, "U")
		cr.DS("tx_delay_sum", "COUNTER", interval*2, 0, "U")
		cr.DS("received_proposer", "COUNTER", interval*2, 0, "U")
		cr.DS("received_voter", "COUNTER", interval*2, 0, "U")
		cr.DS("received_tx", "COUNTER", interval*2, 0, "U")
		cr.DS("confirmed_tx_blk", "COUNTER", interval*2, 0, "U")
		cr.DS("deconfirmed_tx_blk", "COUNTER", interval*2, 0, "U")
		cr.DS("txblk_cfm_sum", "COUNTER", interval*2, 0, "U")
		cr.DS("prop_chain_depth", "COUNTER", interval*2, 0, "U")
		cr.DS("voter_chains_depth", "COUNTER", interval*2, 0, "U")
		cr.DS("processed_proposer", "COUNTER", interval*2, 0, "U")
		cr.DS("processed_voter", "COUNTER", interval*2, 0, "U")
		cr.DS("proposer_delay_mean", "COMPUTE", "proposer_delay_sum,received_proposer,/")
		cr.DS("voter_delay_mean", "COMPUTE", "voter_delay_sum,received_voter,/")
		cr.DS("tx_delay_mean", "COMPUTE", "tx_delay_sum,received_tx,/")
		cr.DS("txblk_cfm_mean", "COMPUTE", "txblk_cfm_sum,confirmed_tx_blk,/,1000,/")
		cr.DS("prop_fork", "COMPUTE", "processed_proposer,prop_chain_depth,-,processed_proposer,/")
		cr.DS("voter_fork", "COMPUTE", "processed_voter,voter_chains_depth,-,processed_voter,/")
		cr.RRA("LAST", 0, 1, duration/interval)
		err = cr.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
		n = path.Clean(dataDir + "/accumulative.rrd")
		cr = rrd.NewCreator(n, time.Now(), interval)
		cr.DS("confirmed_tx", "GAUGE", interval*2, 0, "U")
		cr.DS("deconfirmed_tx", "GAUGE", interval*2, 0, "U")
		cr.DS("generated_tx", "GAUGE", interval*2, 0, "U")
		cr.DS("mined_proposer", "GAUGE", interval*2, 0, "U")
		cr.DS("mined_voter", "GAUGE", interval*2, 0, "U")
		cr.DS("mined_tx", "GAUGE", interval*2, 0, "U")
		cr.DS("proposer_delay_sum", "GAUGE", interval*2, 0, "U")
		cr.DS("voter_delay_sum", "GAUGE", interval*2, 0, "U")
		cr.DS("tx_delay_sum", "GAUGE", interval*2, 0, "U")
		cr.DS("received_proposer", "GAUGE", interval*2, 0, "U")
		cr.DS("received_voter", "GAUGE", interval*2, 0, "U")
		cr.DS("received_tx", "GAUGE", interval*2, 0, "U")
		cr.DS("confirmed_tx_blk", "GAUGE", interval*2, 0, "U")
		cr.DS("deconfirmed_tx_blk", "GAUGE", interval*2, 0, "U")
		cr.DS("txblk_cfm_sum", "GAUGE", interval*2, 0, "U")
		cr.DS("prop_chain_depth", "GAUGE", interval*2, 0, "U")
		cr.DS("voter_chains_depth", "GAUGE", interval*2, 0, "U")
		cr.DS("processed_proposer", "GAUGE", interval*2, 0, "U")
		cr.DS("processed_voter", "GAUGE", interval*2, 0, "U")
		cr.DS("proposer_delay_mean", "COMPUTE", "proposer_delay_sum,received_proposer,/")
		cr.DS("voter_delay_mean", "COMPUTE", "voter_delay_sum,received_voter,/")
		cr.DS("tx_delay_mean", "COMPUTE", "tx_delay_sum,received_tx,/")
		cr.DS("txblk_cfm_mean", "COMPUTE", "txblk_cfm_sum,confirmed_tx_blk,/,1000,/")
		cr.DS("prop_fork", "COMPUTE", "processed_proposer,prop_chain_depth,-,processed_proposer,/")
		cr.DS("voter_fork", "COMPUTE", "processed_voter,voter_chains_depth,-,processed_voter,/")
		cr.RRA("LAST", 0, 1, duration/interval)
		err = cr.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
	}

	fmt.Println("Start logging data")
	c := make(chan Report)
	for k, v := range nodes {
		monitor(k, v, interval, c)
	}

	// start goroutine to carry out the experiment
	expStateUpdate := make(chan bool)
	var expStopAlarm <-chan time.Time
	expStarted := false
	expRunning := false
	expStopped := false
	expStartTime := 0
	var expStartPerf expSnapshot
	var expStopPerf expSnapshot
	var lastSnapshot expSnapshot
	exec.Command("stty", "-F", "/dev/tty", "cbreak").Run()
	exec.Command("stty", "-F", "/dev/tty", "-echo").Run()
	go func() {
		var b []byte = make([]byte, 1)
		for {
			os.Stdin.Read(b)
			if string(b) == "x" {
				expStateUpdate <- true
			}
		}
	}()

	// start displaying data and logging aggregated value
	var avgUpdater *rrd.Updater
	var accUpdater *rrd.Updater
	if grafana {
		avgUpdater = rrd.NewUpdater("data/average.rrd")
		accUpdater = rrd.NewUpdater("data/accumulative.rrd")
	}
	t := time.NewTicker(time.Duration(interval) * time.Second)
	prev := make(map[string]Snapshot)
	curr := make(map[string]Snapshot)
	start := time.Now()
	go func() {
		for {
			select {
			case r := <-c:
				cv, cvp := curr[r.Node]
				if cvp {
					prev[r.Node] = cv
				}
				curr[r.Node] = r.Data
			case now := <-t.C:
				if len(curr) == 0 || len(prev) == 0 {
					continue
				}
				// calculate average among nodes
				ctot := Snapshot{}
				for _, v := range curr {
					ctot.Generated_transactions += v.Generated_transactions
					ctot.Confirmed_transactions += v.Confirmed_transactions
					ctot.Deconfirmed_transactions += v.Deconfirmed_transactions
					ctot.Incoming_message_queue += v.Incoming_message_queue
					ctot.Mined_proposer_blocks += v.Mined_proposer_blocks
					ctot.Mined_voter_blocks += v.Mined_voter_blocks
					ctot.Mined_transaction_blocks += v.Mined_transaction_blocks
					ctot.Total_proposer_block_delay += v.Total_proposer_block_delay
					ctot.Total_voter_block_delay += v.Total_voter_block_delay
					ctot.Total_transaction_block_delay += v.Total_transaction_block_delay
					ctot.Received_proposer_blocks += v.Received_proposer_blocks
					ctot.Received_voter_blocks += v.Received_voter_blocks
					ctot.Received_transaction_blocks += v.Received_transaction_blocks
					ctot.Confirmed_transaction_blocks += v.Confirmed_transaction_blocks
					ctot.Deconfirmed_transaction_blocks += v.Deconfirmed_transaction_blocks
					ctot.Total_transaction_block_confirmation_latency += v.Total_transaction_block_confirmation_latency
					ctot.Proposer_main_chain_length += v.Proposer_main_chain_length
					ctot.Voter_main_chain_length_sum += v.Voter_main_chain_length_sum
					ctot.Processed_proposer_blocks += v.Processed_proposer_blocks
					ctot.Processed_voter_blocks += v.Processed_voter_blocks
				}
				cavg := Snapshot{
					Generated_transactions:                       ctot.Generated_transactions,
					Confirmed_transactions:                       ctot.Confirmed_transactions / len(curr),
					Deconfirmed_transactions:                     ctot.Deconfirmed_transactions / len(curr),
					Incoming_message_queue:                       ctot.Incoming_message_queue / len(curr),
					Mined_proposer_blocks:                        ctot.Mined_proposer_blocks,
					Mined_voter_blocks:                           ctot.Mined_voter_blocks,
					Mined_transaction_blocks:                     ctot.Mined_transaction_blocks,
					Total_proposer_block_delay:                   ctot.Total_proposer_block_delay,
					Total_voter_block_delay:                      ctot.Total_voter_block_delay,
					Total_transaction_block_delay:                ctot.Total_transaction_block_delay,
					Received_proposer_blocks:                     ctot.Received_proposer_blocks,
					Received_voter_blocks:                        ctot.Received_voter_blocks,
					Received_transaction_blocks:                  ctot.Received_transaction_blocks,
					Confirmed_transaction_blocks:                 ctot.Confirmed_transaction_blocks / len(curr),
					Deconfirmed_transaction_blocks:               ctot.Deconfirmed_transaction_blocks / len(curr),
					Total_transaction_block_confirmation_latency: ctot.Total_transaction_block_confirmation_latency / len(curr),
					Proposer_main_chain_length:                   ctot.Proposer_main_chain_length,
					Voter_main_chain_length_sum:                  ctot.Voter_main_chain_length_sum,
					Processed_proposer_blocks:                    ctot.Processed_proposer_blocks,
					Processed_voter_blocks:                       ctot.Processed_voter_blocks,
				}
				ptot := Snapshot{}
				for _, v := range prev {
					ptot.Generated_transactions += v.Generated_transactions
					ptot.Confirmed_transactions += v.Confirmed_transactions
					ptot.Deconfirmed_transactions += v.Deconfirmed_transactions
					ptot.Incoming_message_queue += v.Incoming_message_queue
					ptot.Mined_proposer_blocks += v.Mined_proposer_blocks
					ptot.Mined_voter_blocks += v.Mined_voter_blocks
					ptot.Mined_transaction_blocks += v.Mined_transaction_blocks
					ptot.Total_proposer_block_delay += v.Total_proposer_block_delay
					ptot.Total_voter_block_delay += v.Total_voter_block_delay
					ptot.Total_transaction_block_delay += v.Total_transaction_block_delay
					ptot.Received_proposer_blocks += v.Received_proposer_blocks
					ptot.Received_voter_blocks += v.Received_voter_blocks
					ptot.Received_transaction_blocks += v.Received_transaction_blocks
					ptot.Confirmed_transaction_blocks += v.Confirmed_transaction_blocks
					ptot.Deconfirmed_transaction_blocks += v.Deconfirmed_transaction_blocks
					ptot.Total_transaction_block_confirmation_latency += v.Total_transaction_block_confirmation_latency
					ptot.Proposer_main_chain_length += v.Proposer_main_chain_length
					ptot.Voter_main_chain_length_sum += v.Voter_main_chain_length_sum
					ptot.Processed_proposer_blocks += v.Processed_proposer_blocks
					ptot.Processed_voter_blocks += v.Processed_voter_blocks
				}
				pavg := Snapshot{
					Generated_transactions:                       ptot.Generated_transactions,
					Confirmed_transactions:                       ptot.Confirmed_transactions / len(prev),
					Deconfirmed_transactions:                     ptot.Deconfirmed_transactions / len(prev),
					Incoming_message_queue:                       ptot.Incoming_message_queue / len(prev),
					Mined_proposer_blocks:                        ptot.Mined_proposer_blocks,
					Mined_voter_blocks:                           ptot.Mined_voter_blocks,
					Mined_transaction_blocks:                     ptot.Mined_transaction_blocks,
					Total_proposer_block_delay:                   ptot.Total_proposer_block_delay,
					Total_voter_block_delay:                      ptot.Total_voter_block_delay,
					Total_transaction_block_delay:                ptot.Total_transaction_block_delay,
					Received_proposer_blocks:                     ptot.Received_proposer_blocks,
					Received_voter_blocks:                        ptot.Received_voter_blocks,
					Received_transaction_blocks:                  ptot.Received_transaction_blocks,
					Confirmed_transaction_blocks:                 ptot.Confirmed_transaction_blocks / len(prev),
					Deconfirmed_transaction_blocks:               ptot.Deconfirmed_transaction_blocks / len(prev),
					Total_transaction_block_confirmation_latency: ptot.Total_transaction_block_confirmation_latency / len(prev),
					Proposer_main_chain_length:                   ptot.Proposer_main_chain_length,
					Voter_main_chain_length_sum:                  ptot.Voter_main_chain_length_sum,
					Processed_proposer_blocks:                    ptot.Processed_proposer_blocks,
					Processed_voter_blocks:                       ptot.Processed_voter_blocks,
				}
				dur := int(now.Sub(start).Seconds())
				// deal with the experiment
				select {
				case <-expStateUpdate:
					expStarted = true
					expRunning = false
					expStopped = false
					expStartTime = dur
					lastSnapshot = expSnapshot{
						time:               dur,
						confirmed_tx:       curr["node_0"].Confirmed_transactions,
						confirmed_tx_blk:   curr["node_0"].Confirmed_transaction_blocks,
						processed_voter:    curr["node_0"].Processed_voter_blocks,
						processed_proposer: curr["node_0"].Processed_proposer_blocks,
						voter_len_sum:      curr["node_0"].Voter_main_chain_length_sum,
						proposer_len:       curr["node_0"].Proposer_main_chain_length,
						latency_sum:        curr["node_0"].Total_transaction_block_confirmation_latency,
					}
					expStopAlarm = time.After(300 * time.Second)
				case <-expStopAlarm:
					expStarted = true
					expRunning = false
					expStopped = true
				default:
				}

				if expStarted && (!expStopped) {
					if !expRunning {
						// if the round just bumped
						if lastSnapshot.confirmed_tx != curr["node_0"].Confirmed_transactions {
							expRunning = true
							expStartPerf = expSnapshot{
								time:               dur,
								confirmed_tx:       lastSnapshot.confirmed_tx,
								confirmed_tx_blk:   lastSnapshot.confirmed_tx_blk,
								processed_voter:    lastSnapshot.processed_voter,
								processed_proposer: lastSnapshot.processed_proposer,
								voter_len_sum:      lastSnapshot.voter_len_sum,
								proposer_len:       lastSnapshot.proposer_len,
								latency_sum:        lastSnapshot.latency_sum,
							}
						}
					} else {
						// if the round just bumped
						if lastSnapshot.confirmed_tx != curr["node_0"].Confirmed_transactions {
							expStopPerf = expSnapshot{
								time:               dur,
								confirmed_tx:       lastSnapshot.confirmed_tx,
								confirmed_tx_blk:   lastSnapshot.confirmed_tx_blk,
								processed_voter:    lastSnapshot.processed_voter,
								processed_proposer: lastSnapshot.processed_proposer,
								voter_len_sum:      lastSnapshot.voter_len_sum,
								proposer_len:       lastSnapshot.proposer_len,
								latency_sum:        lastSnapshot.latency_sum,
							}
						}
					}
				}

				lastSnapshot = expSnapshot{
					time:               dur,
					confirmed_tx:       curr["node_0"].Confirmed_transactions,
					confirmed_tx_blk:   curr["node_0"].Confirmed_transaction_blocks,
					processed_voter:    curr["node_0"].Processed_voter_blocks,
					processed_proposer: curr["node_0"].Processed_proposer_blocks,
					voter_len_sum:      curr["node_0"].Voter_main_chain_length_sum,
					proposer_len:       curr["node_0"].Proposer_main_chain_length,
					latency_sum:        curr["node_0"].Total_transaction_block_confirmation_latency,
				}

				// display the values
				tm.Clear()
				tm.MoveCursor(1, 1)
				tm.Printf("Experiment duration - %v sec\n", dur)
				tm.Printf("                                  %8v  %8v\n", "Overall", "Window")
				tm.Printf("        Generated Transactions    %8v  %8v\n", cavg.Generated_transactions/dur, (cavg.Generated_transactions-pavg.Generated_transactions)/int(interval))
				tm.Printf("        Confirmed Transactions    %8v  %8v\n", cavg.Confirmed_transactions/dur, (cavg.Confirmed_transactions-pavg.Confirmed_transactions)/int(interval))
				tm.Printf("      Deconfirmed Transactions    %8v  %8v\n", cavg.Deconfirmed_transactions/dur, (cavg.Deconfirmed_transactions-pavg.Deconfirmed_transactions)/int(interval))
				tm.Printf("  Confirmed Transaction Blocks    %8v  %8v\n", cavg.Confirmed_transaction_blocks/dur, (cavg.Confirmed_transaction_blocks-pavg.Confirmed_transaction_blocks)/int(interval))
				tm.Printf("Deconfirmed Transaction Blocks    %8v  %8v\n", cavg.Deconfirmed_transaction_blocks/dur, (cavg.Deconfirmed_transaction_blocks-pavg.Deconfirmed_transaction_blocks)/int(interval))
				tm.Printf("                  Queue Length    %8v  %8v\n", cavg.Incoming_message_queue, (cavg.Incoming_message_queue-pavg.Incoming_message_queue)/int(interval))
				tm.Printf("          Mining -    Proposer    %8.3g  %8.3g\n", float64(cavg.Mined_proposer_blocks)/float64(dur), float64(cavg.Mined_proposer_blocks-pavg.Mined_proposer_blocks)/float64(interval))
				tm.Printf("          Mining -       Voter    %8.3g  %8.3g\n", float64(cavg.Mined_voter_blocks)/float64(dur), float64(cavg.Mined_voter_blocks-pavg.Mined_voter_blocks)/float64(interval))
				tm.Printf("          Mining - Transaction    %8.3g  %8.3g\n", float64(cavg.Mined_transaction_blocks)/float64(dur), float64(cavg.Mined_transaction_blocks-pavg.Mined_transaction_blocks)/float64(interval))
				tm.Printf("           Delay -    Proposer    %8.3g  %8.3g\n", float64(cavg.Total_proposer_block_delay)/float64(cavg.Received_proposer_blocks), float64(cavg.Total_proposer_block_delay-pavg.Total_proposer_block_delay)/float64(cavg.Received_proposer_blocks-pavg.Received_proposer_blocks))
				tm.Printf("           Delay -       Voter    %8.3g  %8.3g\n", float64(cavg.Total_voter_block_delay)/float64(cavg.Received_voter_blocks), float64(cavg.Total_voter_block_delay-pavg.Total_voter_block_delay)/float64(cavg.Received_voter_blocks-pavg.Received_voter_blocks))
				tm.Printf("           Delay - Transaction    %8.3g  %8.3g\n", float64(cavg.Total_transaction_block_delay)/float64(cavg.Received_transaction_blocks), float64(cavg.Total_transaction_block_delay-pavg.Total_transaction_block_delay)/float64(cavg.Received_transaction_blocks-pavg.Received_transaction_blocks))
				tm.Printf("    Confirmation -       Block    %8.3g  %8.3g\n", float64(cavg.Total_transaction_block_confirmation_latency)/float64(cavg.Confirmed_transaction_blocks)/1000.0, float64(cavg.Total_transaction_block_confirmation_latency-pavg.Total_transaction_block_confirmation_latency)/float64(cavg.Confirmed_transaction_blocks-pavg.Confirmed_transaction_blocks)/1000.0)
				tm.Printf("         Forking -    Proposer    %8.3g  %8.3g\n", float64(cavg.Processed_proposer_blocks-cavg.Proposer_main_chain_length)/float64(cavg.Processed_proposer_blocks), float64((cavg.Processed_proposer_blocks-cavg.Proposer_main_chain_length)-(pavg.Processed_proposer_blocks-pavg.Proposer_main_chain_length))/float64(cavg.Processed_proposer_blocks-pavg.Processed_proposer_blocks))
				tm.Printf("         Forking -       Voter    %8.3g  %8.3g\n", float64(cavg.Processed_voter_blocks-cavg.Voter_main_chain_length_sum)/float64(cavg.Processed_voter_blocks), float64((cavg.Processed_voter_blocks-cavg.Voter_main_chain_length_sum)-(pavg.Processed_voter_blocks-pavg.Voter_main_chain_length_sum))/float64(cavg.Processed_voter_blocks-pavg.Processed_voter_blocks))
				if expStopped {
					if expStarted {
						expdur := expStopPerf.time - expStartPerf.time
						tm.Printf("Experiment Result\n")
						tm.Printf("Time         %7v\n", expdur)
						tm.Printf("Cfm Ltcy     %7.7g\n", float64(expStopPerf.latency_sum-expStartPerf.latency_sum)/float64(expStopPerf.confirmed_tx_blk-expStartPerf.confirmed_tx_blk)/1000.0)
						tm.Printf("Thruput      %7.7g\n", float64(expStopPerf.confirmed_tx-expStartPerf.confirmed_tx)/float64(expdur))
						tm.Printf("Prop Fork    %7.7g\n", float64(expStopPerf.processed_proposer-expStopPerf.proposer_len-expStartPerf.processed_proposer+expStartPerf.proposer_len)/float64(expStopPerf.processed_proposer-expStartPerf.processed_proposer))
						tm.Printf("Vote Fork    %7.7g\n", float64(expStopPerf.processed_voter-expStopPerf.voter_len_sum-expStartPerf.processed_voter+expStartPerf.voter_len_sum)/float64(expStopPerf.processed_voter-expStartPerf.processed_voter))
					}
				} else {
					if !expStarted {
						tm.Printf("Hit x to start a recording\n")
					} else {
						tm.Printf("Experiment running. Remaining time: %v seconds\n", 300-dur+expStartTime)
					}
				}
				tm.Flush()
				// log the aggregated value
				if grafana {
					err = avgUpdater.Update(time.Now(), cavg.Confirmed_transactions, cavg.Deconfirmed_transactions, cavg.Generated_transactions, cavg.Incoming_message_queue, cavg.Mined_proposer_blocks, cavg.Mined_voter_blocks, cavg.Mined_transaction_blocks, cavg.Total_proposer_block_delay, cavg.Total_voter_block_delay, cavg.Total_transaction_block_delay, cavg.Received_proposer_blocks, cavg.Received_voter_blocks, cavg.Received_transaction_blocks, cavg.Confirmed_transaction_blocks, cavg.Deconfirmed_transaction_blocks, cavg.Total_transaction_block_confirmation_latency, cavg.Proposer_main_chain_length, cavg.Voter_main_chain_length_sum, cavg.Processed_proposer_blocks, cavg.Processed_voter_blocks)
					if err != nil {
						// sometimes we get error if interval is set to 1 and the timer goes a bit faster
						continue
					}
					err = accUpdater.Update(time.Now(), cavg.Confirmed_transactions, cavg.Deconfirmed_transactions, cavg.Generated_transactions, cavg.Mined_proposer_blocks, cavg.Mined_voter_blocks, cavg.Mined_transaction_blocks, cavg.Total_proposer_block_delay, cavg.Total_voter_block_delay, cavg.Total_transaction_block_delay, cavg.Received_proposer_blocks, cavg.Received_voter_blocks, cavg.Received_transaction_blocks, cavg.Confirmed_transaction_blocks, cavg.Deconfirmed_transaction_blocks, cavg.Total_transaction_block_confirmation_latency, cavg.Proposer_main_chain_length, cavg.Voter_main_chain_length_sum, cavg.Processed_proposer_blocks, cavg.Processed_voter_blocks)
					if err != nil {
						// sometimes we get error if interval is set to 1 and the timer goes a bit faster
						continue
					}
				}
			}
		}
	}()

	select {}
}

func monitor(node string, url string, interval uint, datachan chan Report) {
	ticker := time.NewTicker(time.Duration(interval) * time.Second)
	updater := rrd.NewUpdater("data/" + node + ".rrd")
	go func() {
		for range ticker.C {
			resp, err := http.Get(url)
			if err != nil {
				continue // the node is not running yet
			}
			defer resp.Body.Close()

			snapshot := Snapshot{}
			err = json.NewDecoder(resp.Body).Decode(&snapshot)
			if err != nil {
				continue
			}
			err = updater.Update(time.Now(), snapshot.Confirmed_transactions, snapshot.Deconfirmed_transactions, snapshot.Generated_transactions, snapshot.Incoming_message_queue, snapshot.Mined_proposer_blocks, snapshot.Mined_voter_blocks, snapshot.Mined_transaction_blocks, snapshot.Total_proposer_block_delay, snapshot.Total_voter_block_delay, snapshot.Total_transaction_block_delay, snapshot.Received_proposer_blocks, snapshot.Received_voter_blocks, snapshot.Received_transaction_blocks, snapshot.Confirmed_transaction_blocks, snapshot.Deconfirmed_transaction_blocks, snapshot.Total_transaction_block_confirmation_latency, snapshot.Proposer_main_chain_length, snapshot.Voter_main_chain_length_sum, snapshot.Processed_proposer_blocks, snapshot.Processed_voter_blocks)
			if err != nil {
				// sometimes we get error if interval is set to 1 and the timer goes a bit faster
				continue
			}
			datachan <- Report{Node: node, Data: snapshot}
		}
	}()
}
