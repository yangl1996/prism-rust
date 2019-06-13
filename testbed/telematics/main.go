package main

import (
	"github.com/ziutek/rrd"
	"fmt"
	"time"
	"os"
)

func main() {
	c := rrd.NewCreator("data.rrd", time.Now(), 10)
	c.DS("confirmed_tx", "COUNTER", 20, 0, "U")
	err := c.Create(true)
	if err != nil {
		fmt.Println("Error creating round-robin database:", err)
		os.Exit(1)
	}
}
