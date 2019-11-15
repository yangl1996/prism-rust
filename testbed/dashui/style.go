package main

import (
	"github.com/wcharczuk/go-chart"
	"github.com/wcharczuk/go-chart/drawing"
)

func DefaultTimeSeries(w, h int, s, dpi float64, title string) *Figure {
	c := Figure{}
	c.Width = int(float64(w) * s)
	c.Height = int(float64(h) * s)
	c.DPI = dpi
	c.Background = chart.Style{
		Padding: chart.Box{
			Top:    50,
			Left:   10,
			Right:  10,
			Bottom: 10,
		},
		FillColor: drawing.ColorFromHex("efefef"),
	}
	c.YAxis = chart.YAxis{
		Style: chart.Style{
			Show: true,
		},
	}
	c.FigureTitle = title
	return &c
}
