package main

import (
	"log"
	"os"
	"time"

	"github.com/hajimehoshi/ebiten"
)

const w, h = 960, 540

func main() {
	s := ebiten.DeviceScaleFactor()
	ebiten.SetRunnableInBackground(true)

	g := DefaultTimeSeries(w, h, s)
	proposerSeries := TimeSeries{}
	proposerSeries.Consolidation = Avg
	proposerSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	voterSeries := TimeSeries{}
	voterSeries.Consolidation = Avg
	voterSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	transactionSeries := TimeSeries{}
	transactionSeries.Consolidation = Avg
	transactionSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	ds := []Dataset{&proposerSeries, &voterSeries, &transactionSeries}

	m := g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-60) * time.Second), time.Now())

	go func() {
		extractDelay("../0.log", &proposerSeries, &voterSeries, &transactionSeries)
	}()

	go func() {
		for range time.NewTicker(8 * time.Millisecond).C {
			m = g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-60) * time.Second), time.Now())
		}
	}()

	update := func(screen *ebiten.Image) error {
		if ebiten.IsKeyPressed(ebiten.KeyEscape) || ebiten.IsKeyPressed(ebiten.KeyQ) {
			os.Exit(0)
		}

		if !ebiten.IsDrawingSkipped() {
			screen.ReplacePixels(m.Pix)
		}

		return nil
	}

	if err := ebiten.Run(update, int(w*s), int(h*s), 1/s, "Ebiten + go-chart"); err != nil {
		log.Fatal(err)
	}
}

