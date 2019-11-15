package main

import (
	"flag"
	"log"
	"os"
	"time"

	"github.com/hajimehoshi/ebiten"
)

func dashboard(args []string) {
	cmd := flag.NewFlagSet("dashboard", flag.ExitOnError)
	widthFlag := cmd.Int("width", 970, "width of the visualization window")
	heightFlag := cmd.Int("height", 600, "height of the visualization window")
	logFlag := cmd.String("log", "../0.log", "path to the Prism client log file")
	cmd.Parse(args)

	w := *widthFlag
	h := *heightFlag

	s := ebiten.DeviceScaleFactor()
	ebiten.SetRunnableInBackground(true)

	// set up figures and datasets
	g := DefaultTimeSeries(w/2, h/2, s, "Block Propagation Delay")
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

	m := g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-60)*time.Second), time.Now())

	// update the datasets
	go func() {
		extractDelay(*logFlag, &proposerSeries, &voterSeries, &transactionSeries)
	}()

	// update the figures
	go func() {
		for range time.NewTicker(8 * time.Millisecond).C {
			m = g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-60)*time.Second), time.Now())
		}
	}()

	update := func(screen *ebiten.Image) error {
		if ebiten.IsKeyPressed(ebiten.KeyEscape) || ebiten.IsKeyPressed(ebiten.KeyQ) {
			os.Exit(0)
		}

		if !ebiten.IsDrawingSkipped() {
			// draw four figures: upper left, upper right, lower left, lower right
			plotUL, _ := ebiten.NewImageFromImage(m, ebiten.FilterNearest)
			plotUR, _ := ebiten.NewImageFromImage(m, ebiten.FilterNearest)
			plotLL, _ := ebiten.NewImageFromImage(m, ebiten.FilterNearest)
			plotLR, _ := ebiten.NewImageFromImage(m, ebiten.FilterNearest)
			optsUL := &ebiten.DrawImageOptions{}
			optsUR := &ebiten.DrawImageOptions{}
			optsUR.GeoM.Translate(float64(w), 0)
			optsLL := &ebiten.DrawImageOptions{}
			optsLL.GeoM.Translate(0, float64(h))
			optsLR := &ebiten.DrawImageOptions{}
			optsLR.GeoM.Translate(float64(w), float64(h))
			screen.DrawImage(plotUL, optsUL)
			screen.DrawImage(plotUR, optsUR)
			screen.DrawImage(plotLL, optsLL)
			screen.DrawImage(plotLR, optsLR)
		}

		return nil
	}

	if err := ebiten.Run(update, int(float64(w)*s), int(float64(h)*s), 1/s, "Prism"); err != nil {
		log.Fatal(err)
	}
}
