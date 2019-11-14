package main

import (
	"image"
	"time"
	"github.com/wcharczuk/go-chart"
)

type Figure struct {
	chart.Chart
}

func (c *Figure) PlotTimeSeries(ds []Dataset, start, end time.Time) *image.RGBA {
	allSeries := []chart.Series{}
	for i, d := range ds {
		time, val := d.Range(start, end)
		series := chart.TimeSeries {
			XValues: time,
			YValues: val,
			Style: chart.Style {
				Show: true,
				StrokeColor: chart.GetDefaultColor(i).WithAlpha(96),
				FillColor: chart.GetDefaultColor(i).WithAlpha(32),
			},
		}
		allSeries = append(allSeries, series)
	}
	c.Series = allSeries
	iw := &chart.ImageWriter{}
	c.Render(chart.PNG, iw)
	m, _ := iw.Image()
	return m.(*image.RGBA)
}
