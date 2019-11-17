package main

import (
	"flag"
	"log"
	"os"
	"time"
	"image"
	"image/color"

	"github.com/hajimehoshi/ebiten"
	"github.com/golang/freetype/truetype"
	"golang.org/x/image/font"
	"github.com/hajimehoshi/ebiten/text"
	"github.com/wcharczuk/go-chart/roboto"
)

func dashboard(args []string) {
	cmd := flag.NewFlagSet("dashboard", flag.ExitOnError)
	widthFlag := cmd.Int("width", 970, "width of the visualization window")
	heightFlag := cmd.Int("height", 600, "height of the visualization window")
	logFlag := cmd.String("log", "../0.log", "path to the Prism client log file")
	dpiFlag := cmd.Int("dpi", 150, "DPI of the plots")
	fpsFlag := cmd.Int("fps", 10, "FPS of the GUI")
	spanFlag := cmd.Int("span", 60, "timespan of the plots")
	cmd.Parse(args)

	if *fpsFlag < 10 {
		log.Fatalf("FPS %v is too low. Set it to at least 10.", *fpsFlag)
	}

	interval := time.Duration(1000000 / *fpsFlag) * time.Microsecond
	w := *widthFlag
	h := *heightFlag
	dpi := float64(*dpiFlag)
	s := ebiten.DeviceScaleFactor()
	span := *spanFlag

	titleHeight := 25

	// set up fonts
	robotoFont, _ := truetype.Parse(roboto.Roboto)
	titleFont := truetype.NewFace(robotoFont, &truetype.Options {
		Size: 13,
		DPI: dpi,
		Hinting: font.HintingFull,
	})

	// calculate the positions, numbers are before scaling
	/*
	(0, 0)			(w / 2, 0)
	Title UL		Title UR
	(0, t)			(w / 2, t)
	Figure UL		Figure UR
	(0, h / 2)		(w / 2, h / 2)
	Title LL		Title LR
	(0, h / 2 + t)		(w / 2, h / 2 + t)
	Figure LL		Figure LR
	*/
	ebiten.SetRunnableInBackground(true)
	ebiten.SetMaxTPS(*fpsFlag)

	// set up figures and datasets
	proposerSeries := TimeSeries{}
	proposerSeries.Consolidation = Avg
	proposerSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	proposerSeries.Title = "Proposer"
	voterSeries := TimeSeries{}
	voterSeries.Consolidation = Avg
	voterSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	voterSeries.Title = "Voter"
	transactionSeries := TimeSeries{}
	transactionSeries.Consolidation = Avg
	transactionSeries.ConsolidationInterval = time.Duration(250) * time.Millisecond
	transactionSeries.Title = "Transaction"
	ds := []Dataset{&proposerSeries, &voterSeries, &transactionSeries}

	readSeries := TimeSeries{}
	readSeries.Consolidation = Sum
	readSeries.ConsolidationInterval = time.Duration(10) * time.Millisecond
	readSeries.Title = "Read"
	readSeries.Interpolation = FillZero
	writeSeries := TimeSeries{}
	writeSeries.Consolidation = Sum
	writeSeries.ConsolidationInterval = time.Duration(10) * time.Millisecond
	writeSeries.Title = "Write"
	writeSeries.Interpolation = FillZero
	ds2 := []Dataset{&readSeries, &writeSeries}

	chartUL := DefaultTimeSeries(w/2, h/2 - titleHeight, s, dpi, "Block Propagation Delay")
	chartUR := DefaultTimeSeries(w/2, h/2 - titleHeight, s, dpi, "Socket Activity")
	chartLL := DefaultTimeSeries(w/2, h/2 - titleHeight, s, dpi, "Block Propagation Delay")
	chartLR := DefaultTimeSeries(w/2, h/2 - titleHeight, s, dpi, "Block Propagation Delay")
	imgUL := image.NewRGBA(image.Rect(0, 0, 1, 1))
	imgUR := image.NewRGBA(image.Rect(0, 0, 1, 1))
	imgLL := image.NewRGBA(image.Rect(0, 0, 1, 1))
	imgLR := image.NewRGBA(image.Rect(0, 0, 1, 1))

	// update the datasets
	go func() {
		extractLog(*logFlag, &proposerSeries, &voterSeries, &transactionSeries, &readSeries, &writeSeries)
	}()

	// update the figures
	go func() {
		for range time.NewTicker(interval).C {
			imgUL = chartUL.PlotTimeSeries(ds, time.Now().Add(time.Duration(-span)*time.Second), time.Now())
		}
	}()
	go func() {
		for range time.NewTicker(interval).C {
			imgUR = chartUR.PlotTimeSeries(ds2, time.Now().Add(time.Duration(-span)*time.Second), time.Now())
		}
	}()
	go func() {
		for range time.NewTicker(interval).C {
			imgLL = chartLL.PlotTimeSeries(ds, time.Now().Add(time.Duration(-span)*time.Second), time.Now())
		}
	}()
	go func() {
		for range time.NewTicker(interval).C {
			imgLR = chartLR.PlotTimeSeries(ds, time.Now().Add(time.Duration(-span)*time.Second), time.Now())
		}
	}()


	update := func(screen *ebiten.Image) error {
		if ebiten.IsKeyPressed(ebiten.KeyEscape) || ebiten.IsKeyPressed(ebiten.KeyQ) {
			os.Exit(0)
		}

		if !ebiten.IsDrawingSkipped() {
			// draw four figures: upper left, upper right, lower left, lower right
			plotUL, _ := ebiten.NewImageFromImage(imgUL, ebiten.FilterNearest)
			optsUL := &ebiten.DrawImageOptions{}
			optsUL.GeoM.Translate(0, float64(titleHeight) * s)
			plotUR, _ := ebiten.NewImageFromImage(imgUR, ebiten.FilterNearest)
			optsUR := &ebiten.DrawImageOptions{}
			optsUR.GeoM.Translate(float64(w) * s / 2, float64(titleHeight) * s)
			plotLL, _ := ebiten.NewImageFromImage(imgLL, ebiten.FilterNearest)
			optsLL := &ebiten.DrawImageOptions{}
			optsLL.GeoM.Translate(0, float64(h) * s / 2 + float64(titleHeight) * s)
			plotLR, _ := ebiten.NewImageFromImage(imgLR, ebiten.FilterNearest)
			optsLR := &ebiten.DrawImageOptions{}
			optsLR.GeoM.Translate(float64(w) * s / 2, float64(h) * s / 2 + float64(titleHeight) * s)

			// batch the draw commands as much as possible for GPU batching
			// clear the background
			screen.Fill(color.White)
			// draw the figures
			screen.DrawImage(plotUL, optsUL)
			screen.DrawImage(plotUR, optsUR)
			screen.DrawImage(plotLL, optsLL)
			screen.DrawImage(plotLR, optsLR)
			// draw the titles
			text.Draw(screen, chartUL.FigureTitle, titleFont, int(float64(5) * s), int(float64(titleHeight) * 0.66 * s), color.Black)
			text.Draw(screen, chartUR.FigureTitle, titleFont, int(float64(w) * s / 2 + float64(5) * s), int(float64(titleHeight) * 0.66 * s), color.Black)
			text.Draw(screen, chartLL.FigureTitle, titleFont, int(float64(5) * s), int(float64(titleHeight) * 0.66 * s + float64(h) * s / 2), color.Black)
			text.Draw(screen, chartLR.FigureTitle, titleFont, int(float64(w) * s / 2 + float64(5) * s), int(float64(titleHeight) * 0.66 * s + float64(h) * s / 2), color.Black)
		}

		return nil
	}

	if err := ebiten.Run(update, int(float64(w)*s), int(float64(h)*s), 1/s, "Prism"); err != nil {
		log.Fatal(err)
	}
}
