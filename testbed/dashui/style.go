package main

import (
	"github.com/wcharczuk/go-chart"
	"github.com/wcharczuk/go-chart/drawing"
)

func DefaultTimeSeries(w, h int, s float64) *Figure {
	c := Figure{}
	c.Width = int(float64(w) * s)
	c.Height = int(float64(h) * s)
	c.Background = chart.Style{
		Padding: chart.Box{
			Top: 25,
			Left: 25,
			Right: 25,
			Bottom: 25,
		},
		FillColor: drawing.ColorFromHex("efefef"),
	}
	c.YAxis = chart.YAxis {
		Style: chart.Style {
			Show: true,
		},
		Range: &chart.ContinuousRange {
			Max: 1,
			Min: -1,
		},
	}
	return &c
}
