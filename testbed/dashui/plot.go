package main

import (
	"github.com/wcharczuk/go-chart"
	"image"
	"time"
)

type Figure struct {
	chart.Chart
	FigureTitle string
	SMA int
}

func (c *Figure) PlotTimeSeries(ds []Dataset, start, end time.Time) *image.RGBA {
	allSeries := []chart.Series{}
	gMax := 0.0
	// get the max of all data series at the same time
	for i, d := range ds {
		time, val := d.Range(start, end)
		for _, d := range val {
			if d > gMax {
				gMax = d
			}
		}
		series := chart.TimeSeries{
			Name: d.Name(),
			XValues: time,
			YValues: val,
			Style: chart.Style{
				Show:        true,
				StrokeColor: chart.GetDefaultColor(i).WithAlpha(96),
				FillColor:   chart.GetDefaultColor(i).WithAlpha(32),
			},
		}
		allSeries = append(allSeries, series)
		if c.SMA != 0 {
			sma := &chart.SMASeries {
				InnerSeries: series,
				Period: c.SMA,
			}
			allSeries = append(allSeries, sma)
		}
	}
	// https://github.com/wcharczuk/go-chart/blob/master/times.go#L43
	c.XAxis.Range = &chart.ContinuousRange {
		Min: float64(start.UnixNano()),
		Max: float64(end.UnixNano()),
	}

	c.YAxis.Range = &chart.ContinuousRange {
		Min: 0.00,
		Max: float64(int(gMax / 100) * 100 + 100),
	}
	c.Series = allSeries
	c.Elements = []chart.Renderable {
		chart.LegendThin(&c.Chart, chart.Style {
			FontSize: 9.0,
			StrokeWidth: 1.5,
		}),
	}
	iw := &chart.ImageWriter{}
	c.Render(chart.PNG, iw)
	m, _ := iw.Image()
	return m.(*image.RGBA)
}

