package main

import (
	"github.com/wcharczuk/go-chart"
	"github.com/wcharczuk/go-chart/drawing"
)

func DefaultTimeSeries(w, h int, s float64, title string) *Figure {
	c := Figure{}
	c.Width = int(float64(w) * s)
	c.Height = int(float64(h) * s)
	c.Background = chart.Style{
		Padding: chart.Box{
			Top:    45,
			Left:   25,
			Right:  25,
			Bottom: 25,
		},
		FillColor: drawing.ColorFromHex("efefef"),
	}
	c.YAxis = chart.YAxis{
		Style: chart.Style{
			Show: true,
		},
	}
	c.Title = title
	c.TitleStyle = chart.Style{
		Show: true,
	}
	return &c
}
