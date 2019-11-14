package main

import (
	"github.com/hpcloud/tail"
	"log"
	"regexp"
	"strconv"
	"time"
)

func extractDelay(f string, p *TimeSeries, v *TimeSeries, t *TimeSeries) {
	file, err := tail.TailFile(f, tail.Config{Follow: true})
	if err != nil {
		log.Fatal(err)
	}

	proposerRegex := regexp.MustCompile(`Received Proposer block, delay=(\d+) ms`)
	voterRegex := regexp.MustCompile(`Received Voter block, delay=(\d+) ms`)
	transactionRegex := regexp.MustCompile(`Received Transaction block, delay=(\d+) ms`)

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
	}
}
