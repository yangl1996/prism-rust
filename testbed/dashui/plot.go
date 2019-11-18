package main

import (
	"github.com/wcharczuk/go-chart"
	"image"
	"time"
	"math"
)

type Figure struct {
	chart.Chart
	FigureTitle string
	SMA int
	OnlySMA bool
	YRangeStep float64
	Prefetch int
}

func (c *Figure) PlotTimeSeries(ds []Dataset, start, end time.Time) *image.RGBA {
	allSeries := []chart.Series{}
	gMax := 0.0
	// get the max of all data series at the same time
	for i, d := range ds {
		times := []time.Time{}
		vals := []float64{}
		// fetch some timestamps before the starting time
		if c.Prefetch != 0 {
			times, vals = d.Range(start.Add(time.Duration(-c.Prefetch) * time.Second), end)
		} else {
			times, vals = d.Range(start, end)
		}

		series := chart.TimeSeries{
			Name: d.Name(),
			XValues: times,
			YValues: vals,
			Style: chart.Style{
				Show:        true,
				StrokeColor: chart.GetDefaultColor(i).WithAlpha(96),
				FillColor:   chart.GetDefaultColor(i).WithAlpha(32),
			},
		}
		if !c.OnlySMA {
			allSeries = append(allSeries, series)
			for _, d := range vals {
				if d > gMax {
					gMax = d
				}
			}
		}
		if c.SMA != 0 {
			seriesName := ""
			if c.OnlySMA {
				seriesName = d.Name()
			}
			sma := &chart.SMASeries {
				Name: seriesName,
				InnerSeries: series,
				Period: c.SMA,
			}
			allSeries = append(allSeries, sma)
			vlen := sma.Len()
			i := int(math.Ceil(float64(vlen) / 10))
			if i < 0 {
				i = 0
			}
			for ; i < vlen; i++ {
				_, y := sma.GetValues(i)
				if y > gMax {
					gMax = y
				}
			}
		}
	}
	// https://github.com/wcharczuk/go-chart/blob/master/times.go#L43
	c.XAxis.Range = &chart.ContinuousRange {
		Min: float64(start.UnixNano()),
		Max: float64(end.UnixNano()),
	}

	if c.YRangeStep != 0 {
		c.YAxis.Range = &chart.ContinuousRange {
			Min: 0.00,
			Max: float64(int(gMax / c.YRangeStep)) * c.YRangeStep + c.YRangeStep,
		}
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

