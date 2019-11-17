package main

import (
	"github.com/hpcloud/tail"
	"log"
	"regexp"
	"strconv"
	"time"
)

func extractLog(f string, p *TimeSeries, v *TimeSeries, t *TimeSeries, r *TimeSeries, w *TimeSeries) {
	file, err := tail.TailFile(f, tail.Config{Follow: true})
	if err != nil {
		log.Fatal(err)
	}

	proposerRegex := regexp.MustCompile(`Received Proposer block, delay=(\d+) ms`)
	voterRegex := regexp.MustCompile(`Received Voter block, delay=(\d+) ms`)
	transactionRegex := regexp.MustCompile(`Received Transaction block, delay=(\d+) ms`)
	readRegex := regexp.MustCompile(`Read (\d+) bytes from socket`)
	writeRegex := regexp.MustCompile(`Wrote (\d+) bytes to socket`)

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
		rMatch := readRegex.FindStringSubmatch(line)
		if rMatch != nil {
			d, _ := strconv.ParseFloat(rMatch[1], 64)
			r.Record(d, time.Now())
			continue
		}
		wMatch := writeRegex.FindStringSubmatch(line)
		if wMatch != nil {
			d, _ := strconv.ParseFloat(wMatch[1], 64)
			w.Record(d, time.Now())
			continue
		}
	}
}
