package main

import (
	"log"
	"math/rand"
	"os"
	"time"

	"github.com/hajimehoshi/ebiten"
)

const w, h = 960, 540

func main() {
	s := ebiten.DeviceScaleFactor()
	ebiten.SetRunnableInBackground(true)

	g := DefaultTimeSeries(w, h, s)
	d := TimeSeries{}
	d.Consolidation = Avg
	d.ConsolidationInterval = time.Duration(16) * time.Millisecond
	ds := []Dataset{&d}

	go generateRandomData(&d, 8*time.Millisecond)

	m := g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-10) * time.Minute), time.Now())

	go func() {
		for range time.NewTicker(8 * time.Millisecond).C {
			m = g.PlotTimeSeries(ds, time.Now().Add(time.Duration(-10) * time.Second), time.Now())
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

func generateRandomData(s *TimeSeries, d time.Duration) {
	for range time.NewTicker(d).C {
		s.Record(rand.Float64(), time.Now())
	}
}
