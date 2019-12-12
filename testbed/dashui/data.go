package main

import (
	"sort"
	"time"
)

type Dataset interface {
	Range(start, end time.Time) ([]time.Time, []float64)
	Record(val float64, t time.Time)
	Name() string
}

type TimeSeries struct {
	consolidated struct {
		time []time.Time
		val  []float64
	}
	raw struct {
		time []time.Time
		val  []float64
	}
	Consolidation         func(vals []float64) float64
	ConsolidationInterval time.Duration
	Interpolation         func() float64
	nextConsolidation     time.Time
	inited                bool
	Title string
}

func (d *TimeSeries) Name() string {
	return d.Title
}

func (d *TimeSeries) Record(val float64, t time.Time) {
	if !d.inited {
		d.nextConsolidation = time.Now().Add(d.ConsolidationInterval)
		d.inited = true
	}
	// check if there's anything that we can consolidate
	if t.After(d.nextConsolidation) {
		// find the timestamp for this consolidation
		cTime := d.nextConsolidation
		for {
			// find the first tick that is not earlier than the last datapoint
			if !d.raw.time[len(d.raw.time)-1].After(cTime) {
				break
			} else {
				if d.Interpolation != nil {
					d.consolidated.time = append(d.consolidated.time, cTime)
					d.consolidated.val = append(d.consolidated.val, d.Interpolation())
				}
			}
			cTime = cTime.Add(d.ConsolidationInterval)
		}

		// note that the raw arries will never be empty
		cVal := d.Consolidation(d.raw.val)
		d.consolidated.time = append(d.consolidated.time, cTime)
		d.consolidated.val = append(d.consolidated.val, cVal)
		d.raw.time = nil
		d.raw.val = nil
		d.nextConsolidation = cTime.Add(d.ConsolidationInterval)
	}
	d.raw.time = append(d.raw.time, t)
	d.raw.val = append(d.raw.val, val)
}

func (d *TimeSeries) Range(start, end time.Time) ([]time.Time, []float64) {
	// search for the first timestamp T that T >= start
	datalen := len(d.consolidated.time)
	startIdx := sort.Search(datalen, func(i int) bool {
		return start.Before(d.consolidated.time[i])
	})
	endIdx := sort.Search(datalen, func(i int) bool {
		return d.consolidated.time[i].After(end)
	})
	if endIdx > len(d.consolidated.val) {
		endIdx = len(d.consolidated.val)
	}
	// return the subslice
	return d.consolidated.time[startIdx:endIdx], d.consolidated.val[startIdx:endIdx]
}

func Avg(vals []float64) float64 {
	tot := .0
	for _, v := range vals {
		tot += v
	}
	return tot / float64(len(vals))
}

func Sum(vals []float64) float64 {
	tot := .0
	for _, v := range vals {
		tot += v
	}
	return tot
}

func FillZero() float64 {
	return 0.0
}
