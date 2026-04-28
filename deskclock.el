;;; deskclock.el --- High-visibility desk clock and countdown timer  -*- lexical-binding: t; -*-

;; Author: Chad Hedstrom
;; Version: 0.1.0
;; Keywords: tools, clock, timer
;; Package-Requires: ((emacs "30.1"))
;; URL: https://github.com/chadh/deskclock

;;; Commentary:

;; Emacs port of the deskclock terminal application.  Renders a
;; full-buffer scaling digital clock, a Pomodoro-style countdown
;; timer, and a stopwatch using a 5x5 block-character font that grows
;; to fill the window.
;;
;; Run M-x deskclock to start.  Inside the buffer:
;;
;;   q          quit
;;   t          switch to Clock mode
;;   c          switch to Countdown mode
;;   s          switch to Stopwatch mode
;;   SPC        start/pause the active timer (countdown or stopwatch)
;;   r          reset the active timer
;;   up/down    adjust countdown minutes
;;   right/left adjust countdown seconds

;;; Code:

(require 'cl-lib)

(defgroup deskclock nil
  "High-visibility desk clock and countdown timer."
  :group 'tools
  :prefix "deskclock-")

(defcustom deskclock-tick-interval 0.2
  "Seconds between redraws."
  :type 'number
  :group 'deskclock)

(defcustom deskclock-default-duration (* 25 60)
  "Default countdown duration, in seconds."
  :type 'integer
  :group 'deskclock)

(defface deskclock-time-face
  '((t :foreground "white" :weight bold))
  "Face for the clock display.")

(defface deskclock-date-face
  '((t :foreground "yellow"))
  "Face for the date display.")

(defface deskclock-timer-active-face
  '((t :foreground "deep sky blue" :weight bold))
  "Face for a running or paused countdown timer.")

(defface deskclock-timer-idle-face
  '((t :foreground "white" :weight bold))
  "Face for a stopped/reset countdown timer.")

(defface deskclock-stopwatch-active-face
  '((t :foreground "magenta" :weight bold))
  "Face for a running stopwatch.")

(defface deskclock-stopwatch-idle-face
  '((t :foreground "white" :weight bold))
  "Face for a stopped stopwatch.")

(defface deskclock-status-face
  '((t :foreground "gray70"))
  "Face for status text such as `Ends at: ...'.")

(defface deskclock-help-face
  '((t :foreground "gray45"))
  "Face for the bottom command bar.")

(defface deskclock-flash-face
  '((t :background "red"))
  "Face used to flash the buffer when the timer finishes.")

;; Font ---------------------------------------------------------------

(defconst deskclock--glyph-width 5)
(defconst deskclock--glyph-height 5)

(defconst deskclock--font
  '((?0 " ███ " "█   █" "█   █" "█   █" " ███ ")
    (?1 "  █  " " ██  " "  █  " "  █  " "  █  ")
    (?2 " ███ " "    █" " ███ " "█    " " ███ ")
    (?3 " ███ " "    █" " ███ " "    █" " ███ ")
    (?4 "█   █" "█   █" " ███ " "    █" "    █")
    (?5 " ███ " "█    " " ███ " "    █" " ███ ")
    (?6 " ███ " "█    " " ███ " "█   █" " ███ ")
    (?7 " ███ " "    █" "    █" "    █" "    █")
    (?8 " ███ " "█   █" " ███ " "█   █" " ███ ")
    (?9 " ███ " "█   █" " ███ " "    █" " ███ ")
    (?:  "  █  " "     " "  █  " "     " "  █  ")
    (?\s "     " "     " "     " "     " "     ")
    (?,  "     " "     " "     " " █   " " █   ")
    (?A " ███ " "█   █" " ███ " "█   █" "█   █")
    (?B " ███ " "█   █" " ███ " "█   █" " ███ ")
    (?C " ███ " "█    " "█    " "█    " " ███ ")
    (?D "█████" "█   █" "█   █" "█   █" "█████")
    (?E " ███ " "█    " " ███ " "█    " " ███ ")
    (?F " ███ " "█    " " ███ " "█    " "█    ")
    (?G " ███ " "█    " " ███ " "█   █" " ███ ")
    (?H "█   █" "█   █" " ███ " "█   █" "█   █")
    (?I " ███ " "  █  " "  █  " "  █  " " ███ ")
    (?J "    █" "    █" "    █" "█   █" " ███ ")
    (?K "█   █" "█  █ " " ██  " "█  █ " "█   █")
    (?L "█    " "█    " "█    " "█    " " ███ ")
    (?M "█   █" "██ ██" "█ █ █" "█   █" "█   █")
    (?N "█   █" "██  █" "█ █ █" "█  ██" "█   █")
    (?O " ███ " "█   █" "█   █" "█   █" " ███ ")
    (?P " ███ " "█   █" " ███ " "█    " "█    ")
    (?Q " ███ " "█   █" " ███ " "█  █ " " ███ ")
    (?R " ███ " "█   █" " ███ " "█  █ " "█   █")
    (?S " ███ " "█    " " ███ " "    █" " ███ ")
    (?T " ███ " "  █  " "  █  " "  █  " "  █  ")
    (?U "█   █" "█   █" "█   █" "█   █" " ███ ")
    (?V "█   █" "█   █" " █ █ " " █ █ " "  █  ")
    (?W "█   █" "█   █" "█ █ █" "██ ██" "█   █")
    (?X "█   █" " █ █ " "  █  " " █ █ " "█   █")
    (?Y "█   █" " █ █ " "  █  " "  █  " "  █  ")
    (?Z " ███ " "    █" "   █ " "  █  " " ███ "))
  "5x5 glyph font: alist of CHAR -> 5 row strings.")

(defun deskclock--glyph (char)
  "Return the 5 row strings for CHAR, uppercased; nil if unknown."
  (cdr (assq (upcase char) deskclock--font)))

;; State --------------------------------------------------------------

(defconst deskclock--buffer-name "*Desk Clock*")

(defvar deskclock--timer nil
  "The single repeating redraw timer.")

(defvar-local deskclock--mode 'clock
  "Either `clock' or `countdown'.")

(defvar-local deskclock--cd-duration nil
  "Configured/remaining duration in seconds.")

(defvar-local deskclock--cd-initial nil
  "Duration the countdown was last started with.")

(defvar-local deskclock--cd-end-time nil
  "Float-time at which the countdown will reach zero, or nil.")

(defvar-local deskclock--cd-running nil)
(defvar-local deskclock--cd-paused nil)

(defvar-local deskclock--sw-elapsed 0
  "Accumulated stopwatch time across pauses, in seconds.")

(defvar-local deskclock--sw-last-start nil
  "Float-time of the most recent stopwatch start, or nil.")

(defvar-local deskclock--sw-running nil)

(defvar-local deskclock--flash-start nil
  "Float-time at which the finish flash began, or nil.")

(defvar-local deskclock--flash-overlay nil)

;; Countdown logic ----------------------------------------------------

(defun deskclock--cd-remaining ()
  (if deskclock--cd-end-time
      (max 0 (- deskclock--cd-end-time (float-time)))
    deskclock--cd-duration))

(defun deskclock--cd-start ()
  (unless deskclock--cd-running
    (when (> deskclock--cd-duration 0)
      (setq deskclock--cd-initial deskclock--cd-duration))
    (setq deskclock--cd-end-time (+ (float-time) (deskclock--cd-remaining))
          deskclock--cd-running t
          deskclock--cd-paused nil)))

(defun deskclock--cd-pause ()
  (when deskclock--cd-running
    (setq deskclock--cd-duration (deskclock--cd-remaining)
          deskclock--cd-end-time nil
          deskclock--cd-running nil
          deskclock--cd-paused t)))

(defun deskclock--cd-stop ()
  (setq deskclock--cd-end-time nil
        deskclock--cd-running nil
        deskclock--cd-paused nil))

(defun deskclock--cd-finish ()
  (setq deskclock--cd-duration 0
        deskclock--cd-end-time nil
        deskclock--cd-running nil
        deskclock--cd-paused t))

(defun deskclock--cd-finished-p ()
  (and deskclock--cd-running
       (zerop (floor (deskclock--cd-remaining)))))

(defun deskclock--cd-adjust-minutes (delta)
  (deskclock--cd-stop)
  (setq deskclock--cd-duration
        (max 0 (+ deskclock--cd-duration (* delta 60)))))

(defun deskclock--cd-adjust-seconds (delta)
  (deskclock--cd-stop)
  (setq deskclock--cd-duration (max 0 (+ deskclock--cd-duration delta))))

;; Stopwatch logic ----------------------------------------------------

(defun deskclock--sw-current ()
  "Return current elapsed stopwatch time in seconds."
  (if (and deskclock--sw-running deskclock--sw-last-start)
      (+ deskclock--sw-elapsed (- (float-time) deskclock--sw-last-start))
    deskclock--sw-elapsed))

(defun deskclock--sw-start ()
  (unless deskclock--sw-running
    (setq deskclock--sw-running t
          deskclock--sw-last-start (float-time))))

(defun deskclock--sw-pause ()
  (when deskclock--sw-running
    (when deskclock--sw-last-start
      (setq deskclock--sw-elapsed
            (+ deskclock--sw-elapsed
               (- (float-time) deskclock--sw-last-start))))
    (setq deskclock--sw-running nil
          deskclock--sw-last-start nil)))

(defun deskclock--sw-reset ()
  (setq deskclock--sw-elapsed 0
        deskclock--sw-last-start nil
        deskclock--sw-running nil))

;; Rendering ----------------------------------------------------------

(defun deskclock--row-string (row chars)
  "Build the ROW-th glyph row across CHARS, with single-space separators."
  (mapconcat
   (lambda (c)
     (or (nth row (deskclock--glyph c))
         (make-string deskclock--glyph-width ?\s)))
   chars
   " "))

(defun deskclock--center (line width)
  (let* ((len (string-width line))
         (lead (max 0 (/ (- width len) 2))))
    (concat (make-string lead ?\s) line)))

(defun deskclock--pad-vertically (lines height)
  (let* ((n (length lines))
         (need (max 0 (- height n)))
         (top (/ need 2))
         (bot (- need top)))
    (append (make-list top "") lines (make-list bot ""))))

(defun deskclock--render-glyph-block (text width height face)
  "Return a list of HEIGHT lines that render TEXT in scaled glyphs.
Lines are propertized with FACE and centered horizontally within WIDTH
columns.  Falls back to a single centered plain line if even base scale
would not fit."
  (let* ((chars (append text nil))
         (n (length chars))
         (base-w (+ (* n deskclock--glyph-width) (max 0 (1- n))))
         (base-h deskclock--glyph-height)
         (scale-w (if (zerop base-w) 1 (/ width base-w)))
         (scale-h (if (zerop base-h) 1 (/ height base-h)))
         (scale (max 1 (min scale-w scale-h)))
         (scaled-w (* base-w scale))
         (scaled-h (* base-h scale)))
    (if (or (> scaled-w width) (> scaled-h height))
        (deskclock--pad-vertically
         (list (deskclock--center (propertize text 'face face) width))
         height)
      (let (lines)
        (dotimes (br base-h)
          (let* ((row (deskclock--row-string br chars))
                 (scaled-row
                  (mapconcat (lambda (c) (make-string scale c))
                             (append row nil) "")))
            (dotimes (_ scale)
              (push (deskclock--center
                     (propertize scaled-row 'face face) width)
                    lines))))
        (deskclock--pad-vertically (nreverse lines) height)))))

(defun deskclock--render-status-block (text width height face)
  (deskclock--pad-vertically
   (list (deskclock--center (propertize text 'face face) width))
   height))

(defun deskclock--format-duration (seconds &optional always-hours)
  (let* ((s (floor seconds))
         (h (/ s 3600))
         (m (% (/ s 60) 60))
         (sec (% s 60)))
    (if (or always-hours (> h 0))
        (format "%02d:%02d:%02d" h m sec)
      (format "%02d:%02d" m sec))))

(defun deskclock--bottom-text ()
  (pcase deskclock--mode
    ('clock "q: Quit | c: Countdown | s: Stopwatch")
    ('countdown
     (if deskclock--cd-running
         "q: Quit | t: Time | s: Stopwatch | SPC: Pause | r: Reset"
       "q: Quit | t: Time | s: Stopwatch | SPC: Start | r: Reset | up/down: Min | left/right: Sec"))
    ('stopwatch
     (if deskclock--sw-running
         "q: Quit | t: Time | c: Countdown | SPC: Pause | r: Reset"
       "q: Quit | t: Time | c: Countdown | SPC: Start | r: Reset"))))

(defun deskclock--build-content (width height-time height-mid height-cmd)
  (let (top mid)
    (pcase deskclock--mode
      ('clock
       (let ((time-str (format-time-string "%I:%M:%S %p"))
             (date-str (upcase (format-time-string "%A, %B %d, %Y"))))
         (setq top (deskclock--render-glyph-block
                    time-str width height-time 'deskclock-time-face))
         (setq mid (deskclock--render-glyph-block
                    date-str width height-mid 'deskclock-date-face))))
      ('countdown
       (let* ((rem (deskclock--cd-remaining))
              (timer-str (deskclock--format-duration rem))
              (face (if (or deskclock--cd-running deskclock--cd-paused)
                        'deskclock-timer-active-face
                      'deskclock-timer-idle-face))
              (visible (cond
                        (deskclock--cd-running t)
                        (deskclock--cd-paused
                         (zerop (% (floor (* 2 (float-time))) 2)))
                        (t t)))
              (status (cond
                       (deskclock--cd-running
                        (format-time-string
                         "Ends at: %I:%M:%S %p"
                         (time-add (current-time) (seconds-to-time rem))))
                       (deskclock--cd-paused "Paused")
                       (t " "))))
         (setq top (deskclock--render-glyph-block
                    (if visible timer-str " ") width height-time face))
         (setq mid (deskclock--render-status-block
                    status width height-mid 'deskclock-status-face))))
      ('stopwatch
       (let* ((elapsed (deskclock--sw-current))
              (timer-str (deskclock--format-duration elapsed t))
              (face (if deskclock--sw-running
                        'deskclock-stopwatch-active-face
                      'deskclock-stopwatch-idle-face))
              (visible (cond
                        (deskclock--sw-running t)
                        ((and (not deskclock--sw-running)
                              (> deskclock--sw-elapsed 0))
                         (zerop (% (floor (* 2 (float-time))) 2)))
                        (t t))))
         (setq top (deskclock--render-glyph-block
                    (if visible timer-str " ") width height-time face))
         (setq mid (deskclock--render-status-block
                    "Stopwatch" width height-mid 'deskclock-status-face)))))
    (let ((bottom (deskclock--render-status-block
                   (deskclock--bottom-text) width height-cmd
                   'deskclock-help-face)))
      (mapconcat #'identity (append top mid bottom) "\n"))))

(defun deskclock--update-flash-overlay ()
  (let ((active
         (and deskclock--flash-start
              (let ((elapsed (* 1000 (- (float-time) deskclock--flash-start))))
                (cond
                 ((>= elapsed 1250)
                  (setq deskclock--flash-start nil)
                  nil)
                 ((zerop (% (/ (floor elapsed) 250) 2)) t)
                 (t nil))))))
    (when (overlayp deskclock--flash-overlay)
      (delete-overlay deskclock--flash-overlay)
      (setq deskclock--flash-overlay nil))
    (when active
      (let ((ov (make-overlay (point-min) (point-max))))
        (overlay-put ov 'face 'deskclock-flash-face)
        (setq deskclock--flash-overlay ov)))))

(defun deskclock--redraw ()
  (let ((buf (get-buffer deskclock--buffer-name)))
    (when (buffer-live-p buf)
      (with-current-buffer buf
        (let ((win (get-buffer-window buf t)))
          (when win
            (let* ((w (window-body-width win))
                   (h (window-body-height win))
                   (h-cmd  (max 1 (/ h 10)))
                   (h-mid  (max 1 (/ (* 2 h) 10)))
                   (h-time (max 1 (- h h-mid h-cmd)))
                   (inhibit-read-only t))
              (when (deskclock--cd-finished-p)
                (setq deskclock--flash-start (float-time))
                (deskclock--cd-finish))
              (erase-buffer)
              (insert (deskclock--build-content w h-time h-mid h-cmd))
              (goto-char (point-min))
              (deskclock--update-flash-overlay))))))))

;; Commands -----------------------------------------------------------

(defun deskclock-quit ()
  "Stop the desk clock timer and kill its buffer."
  (interactive)
  (when (timerp deskclock--timer)
    (cancel-timer deskclock--timer)
    (setq deskclock--timer nil))
  (let ((buf (get-buffer deskclock--buffer-name)))
    (when buf (kill-buffer buf))))

(defun deskclock-switch-to-clock ()
  "Switch to clock mode."
  (interactive)
  (setq deskclock--mode 'clock)
  (deskclock--redraw))

(defun deskclock-switch-to-countdown ()
  "Switch to countdown mode."
  (interactive)
  (setq deskclock--mode 'countdown)
  (deskclock--redraw))

(defun deskclock-switch-to-stopwatch ()
  "Switch to stopwatch mode."
  (interactive)
  (setq deskclock--mode 'stopwatch)
  (deskclock--redraw))

(defun deskclock-toggle-timer ()
  "Start or pause the active timer for the current mode."
  (interactive)
  (pcase deskclock--mode
    ('countdown
     (if deskclock--cd-running (deskclock--cd-pause) (deskclock--cd-start)))
    ('stopwatch
     (if deskclock--sw-running (deskclock--sw-pause) (deskclock--sw-start))))
  (deskclock--redraw))

(defun deskclock-reset-timer ()
  "Reset the active timer for the current mode."
  (interactive)
  (pcase deskclock--mode
    ('countdown
     (deskclock--cd-stop)
     (setq deskclock--cd-duration deskclock--cd-initial))
    ('stopwatch
     (deskclock--sw-reset)))
  (deskclock--redraw))

(defun deskclock-inc-minutes ()
  "Add one minute to the countdown."
  (interactive)
  (deskclock--cd-adjust-minutes 1)
  (deskclock--redraw))

(defun deskclock-dec-minutes ()
  "Subtract one minute from the countdown."
  (interactive)
  (deskclock--cd-adjust-minutes -1)
  (deskclock--redraw))

(defun deskclock-inc-seconds ()
  "Add one second to the countdown."
  (interactive)
  (deskclock--cd-adjust-seconds 1)
  (deskclock--redraw))

(defun deskclock-dec-seconds ()
  "Subtract one second from the countdown."
  (interactive)
  (deskclock--cd-adjust-seconds -1)
  (deskclock--redraw))

(defvar-keymap deskclock-mode-map
  :doc "Keymap for `deskclock-mode'."
  "q"       #'deskclock-quit
  "t"       #'deskclock-switch-to-clock
  "c"       #'deskclock-switch-to-countdown
  "s"       #'deskclock-switch-to-stopwatch
  "SPC"     #'deskclock-toggle-timer
  "r"       #'deskclock-reset-timer
  "<up>"    #'deskclock-inc-minutes
  "<down>"  #'deskclock-dec-minutes
  "<right>" #'deskclock-inc-seconds
  "<left>"  #'deskclock-dec-seconds)

(define-derived-mode deskclock-mode special-mode "DeskClock"
  "Major mode for the desk clock and countdown timer."
  (buffer-disable-undo)
  (setq-local cursor-type nil
              mode-line-format nil
              header-line-format nil
              truncate-lines t
              show-trailing-whitespace nil)
  (setq deskclock--cd-duration deskclock-default-duration
        deskclock--cd-initial  deskclock-default-duration)
  (add-hook 'kill-buffer-hook
            (lambda ()
              (when (timerp deskclock--timer)
                (cancel-timer deskclock--timer)
                (setq deskclock--timer nil)))
            nil t))

;;;###autoload
(defun deskclock ()
  "Open the desk clock in a dedicated buffer."
  (interactive)
  (let ((buf (get-buffer-create deskclock--buffer-name)))
    (with-current-buffer buf
      (unless (derived-mode-p 'deskclock-mode)
        (deskclock-mode)))
    (switch-to-buffer buf)
    (when (timerp deskclock--timer)
      (cancel-timer deskclock--timer))
    (setq deskclock--timer
          (run-with-timer 0 deskclock-tick-interval #'deskclock--redraw))
    (deskclock--redraw)))

(provide 'deskclock)

;;; deskclock.el ends here
