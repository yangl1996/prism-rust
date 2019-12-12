package main

import (
	"github.com/hpcloud/tail"
	"log"
	"regexp"
	"strconv"
	"time"
	"fmt"
	"net/http"
	"encoding/json"
)

type Snapshot struct {
	Confirmed_transactions int
	Proposer_main_chain_length int
	Voter_main_chain_length_sum int
	Processed_proposer_blocks int
	Processed_voter_blocks int
}

func traceCounter(addr string, confirmThroughput *TimeSeries, confirmAmount *TimeSeries, interval time.Duration) {
	url := fmt.Sprintf("http://%v/telematics/snapshot", addr)

	lastQuery := time.Now()
	lastTx := 0

	for range time.NewTicker(interval).C {
		resp, err := http.Get(url)
		thisQuery := time.Now()
		if err != nil {
			continue
		}
		defer resp.Body.Close()

		snapshot := Snapshot{}
		err = json.NewDecoder(resp.Body).Decode(&snapshot)
		if err != nil {
			continue
		}

		timeDiff := thisQuery.Sub(lastQuery).Seconds()
		txDiff := snapshot.Confirmed_transactions - lastTx
		txRate := float64(txDiff) / timeDiff
		lastTx = snapshot.Confirmed_transactions
		confirmThroughput.Record(txRate, thisQuery)
		confirmAmount.Record(float64(snapshot.Confirmed_transactions), thisQuery)

		lastQuery = thisQuery
	}
}

func extractLog(f string, p *TimeSeries, v *TimeSeries, t *TimeSeries, r *TimeSeries, w *TimeSeries, ps *TimeSeries, prs *TimeSeries, pws *TimeSeries, pqs *TimeSeries) {
	file, err := tail.TailFile(f, tail.Config{Follow: true})
	if err != nil {
		log.Fatal(err)
	}

	proposerRegex := regexp.MustCompile(`Received Proposer block, delay=(\d+) ms`)
	voterRegex := regexp.MustCompile(`Received Voter block, delay=(\d+) ms`)
	transactionRegex := regexp.MustCompile(`Received Transaction block, delay=(\d+) ms`)
	readRegex := regexp.MustCompile(`Read (\d+) bytes from socket`)
	writeRegex := regexp.MustCompile(`Wrote (\d+) bytes to socket`)
	pollRegex := regexp.MustCompile(`New polling results received`)
	readableEventRegex := regexp.MustCompile(`Peer (\d+) readable`)
	writableEventRegex := regexp.MustCompile(`Peer (\d+) writable`)
	outqueueEventRegex := regexp.MustCompile(`Peer (\d+) outgoing queue readable`)


	for l := range file.Lines {
		line := l.Text
		pMatch := proposerRegex.FindStringSubmatch(line)
		if pMatch != nil {
			d, _ := strconv.ParseFloat(pMatch[1], 64)
			p.Record(d, time.Now())
			continue
		}
		vMatch := voterRegex.FindStringSubmatch(line)
		if vMatch != nil {
			d, _ := strconv.ParseFloat(vMatch[1], 64)
			v.Record(d, time.Now())
			continue
		}
		tMatch := transactionRegex.FindStringSubmatch(line)
		if tMatch != nil {
			d, _ := strconv.ParseFloat(tMatch[1], 64)
			t.Record(d, time.Now())
			continue
		}
		// all data points below are multiplied by 100
		// because we concatnate every 10ms, 10ms x 100 = 1s
		rMatch := readRegex.FindStringSubmatch(line)
		if rMatch != nil {
			d, _ := strconv.ParseFloat(rMatch[1], 64)
			d = d * 8 * 100 / 1000
			r.Record(d, time.Now())
			continue
		}
		wMatch := writeRegex.FindStringSubmatch(line)
		if wMatch != nil {
			d, _ := strconv.ParseFloat(wMatch[1], 64)
			d = d * 8 * 100 / 1000
			w.Record(d, time.Now())
			continue
		}
		if pollRegex.FindString(line) != "" {
			ps.Record(100, time.Now())
		}
		if readableEventRegex.FindString(line) != "" {
			prs.Record(100, time.Now())
		}
		if writableEventRegex.FindString(line) != "" {
			pws.Record(100, time.Now())
		}
		if outqueueEventRegex.FindString(line) != "" {
			pqs.Record(100, time.Now())
		}
	}
}
