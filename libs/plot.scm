;;; plot.scm -- Simple 2D function plotter for s_rs / srsgtk.
;;;
;;; This library stays in plain Scheme and reuses the srsgtk canvas
;;; primitives (clear-canvas, set-color, draw-line, draw-point) together
;;; with `function-samples' (libs/function-samples.scm).  Load it with
;;;   (load "plot")
;;; once libs/plot.scm is available in the interpreter's load path
;;; (e.g. symlinked/copied to ~/.config/srs/libs/).

(define plot-scale-x
  (lambda (x xmin xmax margin canvas-w)
    "Map data-space X into canvas pixel coordinate, with MARGIN on each
     side.  X and the bounds may be exact or inexact; the result is Float."
    (+ margin
       (* (/ (- canvas-w (* 2 margin)) (- xmax xmin))
          (- x xmin)))))

(define plot-scale-y
  (lambda (y ymin ymax margin canvas-h)
    "Map data-space Y into canvas pixel coordinate.  Inverts the Y axis
     because Cairo's origin is top-left.  Result is Float."
    (- canvas-h
       margin
       (* (/ (- canvas-h (* 2 margin)) (- ymax ymin))
          (- y ymin)))))

(define plot-bounds
  (lambda (segments)
    "Compute (xmin xmax ymin ymax) over all points in SEGMENTS, a list of
     segments returned by function-samples.  Empty input falls back to
     (-1 1 -1 1)."
    (define first-point
      (lambda (segs)
        (if (null? segs)
            #f
            (if (null? (car segs))
                (first-point (cdr segs))
                (car (car segs))))))
    (define scan-segments
      (lambda (segs xmin xmax ymin ymax)
        (if (null? segs)
            (list xmin xmax ymin ymax)
            (scan-points (car segs)
                         (cdr segs)
                         xmin xmax ymin ymax))))
    (define smaller
      (lambda (a b) (if (< a b) a b)))
    (define larger
      (lambda (a b) (if (> a b) a b)))
    (define scan-points
      (lambda (pts segs xmin xmax ymin ymax)
        (if (null? pts)
            (scan-segments segs xmin xmax ymin ymax)
            (let ((x (car (car pts)))
                  (y (cdr (car pts))))
              (scan-points (cdr pts)
                           segs
                           (smaller x xmin)
                           (larger x xmax)
                           (smaller y ymin)
                           (larger y ymax))))))
    (let ((p (first-point segments)))
      (if (not p)
          '(-1.0 1.0 -1.0 1.0)
          (scan-segments segments
                         (car p) (car p)
                         (cdr p) (cdr p))))))

(define plot-axes
  (lambda (xmin xmax ymin ymax)
    "Draw a simple 2D axis frame.  X and Y axes are drawn on 0.0 when
     the interval contains it, otherwise on the nearest border.  Bounds
     and margins are read from current canvas dimensions."
    (let* ((w (canvas-width))
           (h (canvas-height))
           (margin 40)
           (x0 (plot-scale-x (if (< xmin 0 xmax) 0.0 xmin)
                             xmin xmax margin w))
           (y0 (plot-scale-y (if (< ymin 0 ymax) 0.0 ymin)
                             ymin ymax margin h))
           (xmin-px (plot-scale-x xmin xmin xmax margin w))
           (xmax-px (plot-scale-x xmax xmin xmax margin w))
           (ymin-px (plot-scale-y ymin ymin ymax margin h))
           (ymax-px (plot-scale-y ymax ymin ymax margin h)))
      (set-color 0.3 0.3 0.3)
      (draw-line x0 ymax-px x0 ymin-px)
      (draw-line xmin-px y0 xmax-px y0)
      ;; simple ticks along the axis line at the screen edges
      (draw-line xmin-px (- y0 3) xmin-px (+ y0 3))
      (draw-line xmax-px (- y0 3) xmax-px (+ y0 3))
      (draw-line (- x0 3) ymin-px (+ x0 3) ymin-px)
      (draw-line (- x0 3) ymax-px (+ x0 3) ymax-px))))

(define plot-segment
  (lambda (segment xmin xmax ymin ymax margin)
    "Draw a single continuous segment provided as a list of (x . y) pairs."
    (define draw-pairs
      (lambda (pts)
        (if (null? (cdr pts))
            '()
            (let* ((w (canvas-width))
                   (h (canvas-height))
                   (p1 (car pts))
                   (p2 (cadr pts))
                   (x1 (plot-scale-x (car p1) xmin xmax margin w))
                   (y1 (plot-scale-y (cdr p1) ymin ymax margin h))
                   (x2 (plot-scale-x (car p2) xmin xmax margin w))
                   (y2 (plot-scale-y (cdr p2) ymin ymax margin h)))
              (draw-line x1 y1 x2 y2)
              (draw-pairs (cdr pts))))))
    (if (null? segment)
        '()
        (draw-pairs segment))))

(define plot-function
  (lambda (f xmin xmax)
    "Plot the one-argument function F over [XMIN, XMAX].  Sampling points
     are connected within each segment returned by function-samples and
     dropped across discontinuities/asymptotes.  Axes are auto-scaled in Y
     and redrawn on window resize."
    (define redraw
      (lambda (w h)
        (clear-canvas)
        (let* ((segments (function-samples f xmin xmax 400))
               (bounds (plot-bounds segments))
               (xmin-real (car bounds))
               (xmax-real (cadr bounds))
               (ymin-real (caddr bounds))
               (ymax-real (cadddr bounds))
               (margin 40))
          (plot-axes xmin-real xmax-real ymin-real ymax-real)
          (set-color 0.9 0.3 0.2)
          (define draw-all-segments
            (lambda (segs)
              (if (null? segs)
                  '()
                  (begin
                    (plot-segment (car segs)
                                  xmin-real xmax-real
                                  ymin-real ymax-real
                                  margin)
                    (draw-all-segments (cdr segs))))))
          (draw-all-segments segments))))
    (set-redraw-hook! redraw)))
