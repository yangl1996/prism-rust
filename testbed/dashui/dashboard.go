package main

import (
	"log"
	"os"
	"time"
	"flag"

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

	g := DefaultTimeSeries(w / 2, h / 2, s)
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

	go func() {
		extractDelay(*logFlag, &proposerSeries, &voterSeries, &transactionSeries)
	}()

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
			plot1, _ := ebiten.NewImageFromImage(m, ebiten.FilterNearest)
			opts := &ebiten.DrawImageOptions{}
			screen.DrawImage(plot1, opts)
		}

		return nil
	}

	if err := ebiten.Run(update, int(float64(w)*s), int(float64(h)*s), 1/s, "Prism"); err != nil {
		log.Fatal(err)
	}
}
