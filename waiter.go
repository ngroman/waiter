package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"syscall"
	"time"
)

const (
	period = 100 * time.Millisecond
	width  = 80
)

type sleepConf struct {
	Dur     time.Duration
	Text    string
	Say     bool
	WaitPid int
}

func main() {
	conf := sleepConf{}

	flag.BoolVar(&conf.Say, "s", false, "Speak when done")
	flag.IntVar(&conf.WaitPid, "p", 0, "PID for process to wait on")
	flag.Parse()
	argc := flag.NArg()
	var sleepSecs float64
	var err error
	if argc > 0 {
		sleepSecs, err = strconv.ParseFloat(flag.Arg(0), 64)
	}
	if err != nil {
		exitError("sleepSecs must be a number")
	} else if sleepSecs < 0 {
		exitError("sleepSecs must be positive")
	}
	conf.Dur = time.Duration(sleepSecs*1000) * time.Millisecond

	if argc > 1 {
		conf.Text = flag.Arg(1)
	}
	doSleep(conf)
}

func doSleep(conf sleepConf) {
	if pid := conf.WaitPid; pid > 0 {
		fmt.Fprintf(os.Stderr, "Waiting on process %d...", pid)
		waited := waitForProcess(pid)
		if waited {
			fmt.Fprintln(os.Stderr, "DONE")
		} else {
			fmt.Fprintln(os.Stderr, "\nERROR: No such process")
			os.Exit(1)
		}
	}

	if conf.Dur != 0 {
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
			conf.printProgress(remaining)
		}
		conf.printProgress(0)
		fmt.Fprintln(os.Stderr, "")
	}

	alert(conf.Text)
	if conf.Say {
		t := conf.Text
		if t == "" {
			t = "done"
		}
		speakStr(t)
	}
}

func waitForProcess(pid int) bool {
	ticker := time.NewTicker(100 * time.Millisecond)
	waited := false
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			err := syscall.Kill(pid, syscall.Signal(0))
			if err != nil {
				return waited
			}
			waited = true
		}
	}
}

func (conf sleepConf) printProgress(remaining time.Duration) {
	fmt.Fprintf(os.Stderr, "  %s %10s\r", progress(remaining, conf.Dur), fmtDuration(remaining))
}

const (
	barLen int = 20
)

func progress(remaining, total time.Duration) string {
	spaces := int(float64(barLen) * float64(remaining) / float64(total))
	if spaces > barLen {
		spaces = barLen
	} else if spaces < 0 {
		spaces = 0
	}
	return "[" + strings.Repeat("#", barLen-spaces) + strings.Repeat("-", spaces) + "]"
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
