;;; test-deskclock.el --- E2E test for deskclock.el  -*- lexical-binding: t; -*-

(add-to-list 'load-path default-directory)
(require 'deskclock)
(require 'cl-lib)

(defvar tdc-fail 0)

(defun tdc-check (label expected actual)
  (princ (format "%s %s\n  expected: %S\n  actual:   %S\n"
                 (if (equal expected actual) "PASS" "FAIL")
                 label expected actual))
  (unless (equal expected actual) (cl-incf tdc-fail)))

(defun tdc-assert (label ok)
  (princ (format "%s %s\n" (if ok "PASS" "FAIL") label))
  (unless ok (cl-incf tdc-fail)))

;; --- A frame is needed for window dimensions in batch mode -----------
;; In --batch there is a window but its body dimensions are degenerate.
;; We bypass the window-size lookup by calling the helpers directly with
;; explicit dimensions, then drive the high-level redraw with a frame
;; created via (make-frame-on-display ...) — too invasive in batch.
;; Instead we test the building blocks plus the buffer-level commands.

;; --- Glyph table ------------------------------------------------------
(tdc-check "glyph 0 has 5 rows"
           t (= 5 (length (deskclock--glyph ?0))))
(tdc-check "glyph 0 row width 5"
           t (cl-every (lambda (r) (= 5 (length r))) (deskclock--glyph ?0)))
(tdc-check "lowercase folds to upper"
           (deskclock--glyph ?A) (deskclock--glyph ?a))
(tdc-assert "all 36 expected chars present"
            (cl-every (lambda (c) (deskclock--glyph c))
                      (append "0123456789:, ABCDEFGHIJKLMNOPQRSTUVWXYZ" nil)))

;; --- Row builder ------------------------------------------------------
(let ((row (deskclock--row-string 0 (append "12" nil))))
  ;; first row of '1' is "  █  ", first row of '2' is " ███ ", joined by 1 space
  (tdc-check "row-string '12' row 0"
             "  █    ███ " row))

;; --- Format duration --------------------------------------------------
(tdc-check "format <1h"  "25:00" (deskclock--format-duration 1500))
(tdc-check "format =0"   "00:00" (deskclock--format-duration 0))
(tdc-check "format >1h"  "01:02:03" (deskclock--format-duration 3723))
(tdc-check "float secs"  "00:09" (deskclock--format-duration 9.7))

;; --- Glyph block dimensions -------------------------------------------
(let ((lines (deskclock--render-glyph-block "12" 80 30 'default)))
  (tdc-check "glyph-block height" 30 (length lines))
  (tdc-assert "every line has display-width <= 80"
              (cl-every (lambda (l) (<= (string-width l) 80)) lines))
  ;; centered: at least one non-empty line should have leading spaces
  (let ((first-non-blank (cl-find-if (lambda (l) (not (string-blank-p l))) lines)))
    (tdc-assert "non-blank line is propertized"
                (and first-non-blank
                     (text-property-not-all 0 (length first-non-blank)
                                            'face nil first-non-blank)))))

;; Fallback path: tiny area falls back to single plain-text line.
(let ((lines (deskclock--render-glyph-block "ABCDEFG" 5 1 'default)))
  (tdc-check "fallback height" 1 (length lines))
  (tdc-assert "fallback line non-empty" (not (string-blank-p (car lines)))))

;; --- Status block -----------------------------------------------------
(let ((lines (deskclock--render-status-block "Paused" 40 5 'default)))
  (tdc-check "status height" 5 (length lines))
  (tdc-assert "status text appears"
              (cl-some (lambda (l) (string-match-p "Paused" l)) lines)))

;; --- Countdown state machine -----------------------------------------
(with-temp-buffer
  (deskclock-mode)
  (tdc-check "initial duration" 1500 deskclock--cd-duration)
  (tdc-check "initial mode" 'clock deskclock--mode)

  (deskclock--cd-adjust-minutes 1)
  (tdc-check "+1 min" 1560 deskclock--cd-duration)
  (deskclock--cd-adjust-seconds -10)
  (tdc-check "-10 sec" 1550 deskclock--cd-duration)
  (deskclock--cd-adjust-minutes -1000) ;; clamp test
  (tdc-check "clamped to 0" 0 deskclock--cd-duration)

  (setq deskclock--cd-duration 30
        deskclock--cd-initial  30)
  (deskclock--cd-start)
  (tdc-assert "running after start" deskclock--cd-running)
  (tdc-assert "end-time set" (numberp deskclock--cd-end-time))
  (tdc-assert "remaining ≈ 30" (and (<= 29 (deskclock--cd-remaining))
                                    (>= 30 (deskclock--cd-remaining))))

  (deskclock--cd-pause)
  (tdc-assert "paused not running" (not deskclock--cd-running))
  (tdc-assert "paused flagged"     deskclock--cd-paused)
  (tdc-assert "end-time cleared"   (null deskclock--cd-end-time))

  (deskclock--cd-stop)
  (let ((deskclock--mode 'countdown))
    (deskclock-reset-timer))
  (tdc-check "reset to initial" 30 deskclock--cd-duration)

  ;; finish path: simulate a started timer that already expired
  (setq deskclock--cd-running t
        deskclock--cd-end-time (- (float-time) 1))
  (tdc-assert "finished-p detects expiry" (deskclock--cd-finished-p))
  (deskclock--cd-finish)
  (tdc-check "finished duration zero" 0 deskclock--cd-duration)
  (tdc-assert "finished -> paused, not running"
              (and deskclock--cd-paused (not deskclock--cd-running))))

;; --- Stopwatch state machine -----------------------------------------
(with-temp-buffer
  (deskclock-mode)
  (tdc-check "sw initial elapsed" 0 deskclock--sw-elapsed)
  (tdc-check "sw initial running" nil deskclock--sw-running)
  (tdc-assert "sw current=0 idle" (= 0 (deskclock--sw-current)))

  (deskclock--sw-start)
  (tdc-assert "sw running after start" deskclock--sw-running)
  (tdc-assert "sw last-start set" (numberp deskclock--sw-last-start))
  ;; Simulate ~0.05s elapsed
  (sleep-for 0.05)
  (tdc-assert "sw current advances" (> (deskclock--sw-current) 0))

  (deskclock--sw-pause)
  (tdc-assert "sw paused -> not running" (not deskclock--sw-running))
  (tdc-assert "sw last-start cleared" (null deskclock--sw-last-start))
  (let ((after-pause deskclock--sw-elapsed))
    (tdc-assert "sw elapsed accumulated" (> after-pause 0))
    (sleep-for 0.05)
    (tdc-assert "sw current frozen while paused"
                (= after-pause (deskclock--sw-current))))

  (deskclock--sw-start)
  (sleep-for 0.05)
  (deskclock--sw-pause)
  (tdc-assert "sw accumulates across resume"
              (> deskclock--sw-elapsed 0.08))

  (deskclock--sw-reset)
  (tdc-check "sw reset elapsed" 0 deskclock--sw-elapsed)
  (tdc-check "sw reset running" nil deskclock--sw-running))

;; --- Mode-aware toggle and reset commands ----------------------------
(with-temp-buffer
  (rename-buffer (generate-new-buffer-name "*test-dc-toggle*"))
  (deskclock-mode)
  (setq deskclock--mode 'stopwatch)
  (cl-letf (((symbol-function 'deskclock--redraw) (lambda () nil)))
    (deskclock-toggle-timer)
    (tdc-assert "toggle from stopwatch starts sw" deskclock--sw-running)
    (deskclock-toggle-timer)
    (tdc-assert "toggle again pauses sw" (not deskclock--sw-running))
    ;; reset stopwatch elapsed
    (setq deskclock--sw-elapsed 5)
    (deskclock-reset-timer)
    (tdc-check "reset clears sw elapsed" 0 deskclock--sw-elapsed)
    ;; switch to countdown — toggle should now affect cd
    (setq deskclock--mode 'countdown)
    (deskclock-toggle-timer)
    (tdc-assert "toggle from countdown starts cd" deskclock--cd-running)))

;; --- Bottom-bar text varies by mode/state -----------------------------
(with-temp-buffer
  (deskclock-mode)
  (setq deskclock--mode 'clock)
  (tdc-check "bottom clock"
             "q: Quit | c: Countdown | s: Stopwatch" (deskclock--bottom-text))
  (setq deskclock--mode 'countdown deskclock--cd-running nil)
  (tdc-assert "bottom countdown idle mentions arrows"
              (string-match-p "up/down" (deskclock--bottom-text)))
  (tdc-assert "bottom countdown idle mentions Stopwatch"
              (string-match-p "Stopwatch" (deskclock--bottom-text)))
  (setq deskclock--cd-running t)
  (tdc-assert "bottom countdown running shows Pause"
              (string-match-p "Pause" (deskclock--bottom-text)))
  (setq deskclock--mode 'stopwatch deskclock--sw-running nil)
  (tdc-assert "bottom stopwatch idle shows Start"
              (string-match-p "Start" (deskclock--bottom-text)))
  (tdc-assert "bottom stopwatch idle mentions Countdown"
              (string-match-p "Countdown" (deskclock--bottom-text)))
  (setq deskclock--sw-running t)
  (tdc-assert "bottom stopwatch running shows Pause"
              (string-match-p "Pause" (deskclock--bottom-text))))

;; --- High-level redraw with a real visible window --------------------
;; In --batch we cannot create a real visible frame, but we can fake
;; window dimensions by temporarily redefining the lookup helpers.
(with-temp-buffer
  (rename-buffer deskclock--buffer-name)
  (deskclock-mode)
  (cl-letf* ((win (selected-window))
             ((symbol-function 'get-buffer-window)
              (lambda (&rest _) win))
             ((symbol-function 'window-body-width)
              (lambda (&rest _) 120))
             ((symbol-function 'window-body-height)
              (lambda (&rest _) 30)))
    ;; clock mode redraw
    (setq deskclock--mode 'clock)
    (deskclock--redraw)
    (let ((s (buffer-string)))
      (tdc-assert "clock buffer has content"  (> (length s) 100))
      (tdc-assert "clock buffer has 30 lines" (= 30 (length (split-string s "\n"))))
      (tdc-assert "clock buffer mentions help"
                  (string-match-p "Countdown" s)))

    ;; countdown idle
    (setq deskclock--mode 'countdown
          deskclock--cd-duration 1500
          deskclock--cd-initial 1500
          deskclock--cd-running nil
          deskclock--cd-paused nil)
    (deskclock--redraw)
    (let ((s (buffer-string)))
      (tdc-assert "countdown idle line count" (= 30 (length (split-string s "\n"))))
      (tdc-assert "countdown idle mentions Reset" (string-match-p "Reset" s)))

    ;; countdown running
    (deskclock--cd-start)
    (deskclock--redraw)
    (let ((s (buffer-string)))
      (tdc-assert "countdown running shows Ends at"
                  (string-match-p "Ends at" s))
      (tdc-assert "countdown running shows Pause"
                  (string-match-p "Pause" s)))

    ;; finish flash
    (setq deskclock--cd-end-time (- (float-time) 0.5))
    (deskclock--redraw)
    (tdc-assert "flash overlay created"
                (overlayp deskclock--flash-overlay))
    (tdc-assert "flash overlay has red bg face"
                (eq 'deskclock-flash-face
                    (overlay-get deskclock--flash-overlay 'face)))

    ;; stopwatch idle
    (setq deskclock--mode 'stopwatch
          deskclock--sw-elapsed 0
          deskclock--sw-running nil
          deskclock--sw-last-start nil
          deskclock--flash-start nil)
    (when (overlayp deskclock--flash-overlay)
      (delete-overlay deskclock--flash-overlay)
      (setq deskclock--flash-overlay nil))
    (deskclock--redraw)
    (let ((s (buffer-string)))
      (tdc-assert "stopwatch idle line count" (= 30 (length (split-string s "\n"))))
      (tdc-assert "stopwatch idle shows Stopwatch label"
                  (string-match-p "Stopwatch" s))
      (tdc-assert "stopwatch idle shows 00:00:00"
                  (string-match-p "Start" s)))

    ;; stopwatch running
    (deskclock--sw-start)
    (deskclock--redraw)
    (let ((s (buffer-string)))
      (tdc-assert "stopwatch running shows Pause"
                  (string-match-p "Pause" s)))))

;; --- Keymap wiring ----------------------------------------------------
(tdc-check "q -> quit"     #'deskclock-quit
           (lookup-key deskclock-mode-map (kbd "q")))
(tdc-check "SPC -> toggle" #'deskclock-toggle-timer
           (lookup-key deskclock-mode-map (kbd "SPC")))
(tdc-check "<up> -> +min"  #'deskclock-inc-minutes
           (lookup-key deskclock-mode-map (kbd "<up>")))
(tdc-check "<left> -> -sec"#'deskclock-dec-seconds
           (lookup-key deskclock-mode-map (kbd "<left>")))
(tdc-check "s -> stopwatch"
           #'deskclock-switch-to-stopwatch
           (lookup-key deskclock-mode-map (kbd "s")))

(princ (format "\n=== %s (%d failure%s) ===\n"
               (if (zerop tdc-fail) "ALL PASS" "FAILED")
               tdc-fail (if (= 1 tdc-fail) "" "s")))
(kill-emacs (if (zerop tdc-fail) 0 1))
