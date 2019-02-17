package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"
)

const (
	period = 200 * time.Millisecond
	width  = 80
)

type sleepConf struct {
	Dur  time.Duration
	Text string
	Say  bool
}

func main() {
	conf := sleepConf{}

	flag.BoolVar(&conf.Say, "s", false, "Say when done")
	flag.Parse()
	argc := flag.NArg()
	if argc < 1 {
		exitError("Must specify a time")
	}
	sleepSecs, err := strconv.ParseFloat(flag.Arg(0), 64)
	if err != nil {
		exitError("sleepSecs must be a number")
	}
	if sleepSecs < 0 {
		exitError("sleepSecs must be positive")
	}
	conf.Dur = time.Duration(sleepSecs*1000) * time.Millisecond

	if argc > 1 {
		conf.Text = flag.Arg(1)
	}
	doSleep(conf)
}

func doSleep(conf sleepConf) {
	end := time.Now().Add(conf.Dur)
	for {
		now := time.Now()
		if now.After(end) {
			break
		}
		remaining := time.Until(end)
		if remaining < period {
			time.Sleep(remaining)
		} else {
			time.Sleep(period)
		}
		fmt.Fprintf(os.Stderr, "Time remaining: %-10s\r", fmtDuration(remaining))
	}
	fmt.Fprintln(os.Stderr, "")

	alert(conf.Text)
	if conf.Say {
		speakStr(conf.Text)
	}
}

func fmtDuration(d time.Duration) string {
	hrs := ""
	if d > time.Hour {
		hrs = fmt.Sprintf("%02d:", int64(d.Hours()))
	}
	return fmt.Sprintf("%s%02d:%02d", hrs, int64(d/time.Minute)%60, int64(d/time.Second)%60)
}

func speakStr(s string) {
	err := exec.Command("say", s).Run()
	if err != nil {
		beep()
	}
}

func alert(s string) {
	if s == "" {
		s = "Done"
	} else {
		s = strings.Replace(s, "\"", "", -1)
	}
	exec.Command("osascript", "-e", fmt.Sprintf("display notification \"%s\" with title \"waiter\"", s)).Run()
}

func beep() {
	fmt.Fprint(os.Stderr, "\a")
	time.Sleep(600 * time.Millisecond)
	fmt.Fprint(os.Stderr, "\a")
}

func exitError(message string) {
	fmt.Println(message)
	os.Exit(1)
}
