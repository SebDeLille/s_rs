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
    (let ((span (exact->inexact (- xmax xmin)))
          (usable (exact->inexact (- canvas-w (* 2 margin)))))
      (if (= span 0.0)
          (+ margin (* 0.5 usable))
          (+ margin (* (/ usable span) (- x xmin)))))))

(define plot-scale-y
  (lambda (y ymin ymax margin canvas-h)
    "Map data-space Y into canvas pixel coordinate.  Inverts the Y axis
     because Cairo's origin is top-left.  Result is Float."
    (let ((span (exact->inexact (- ymax ymin)))
          (usable (exact->inexact (- canvas-h (* 2 margin)))))
      (if (= span 0.0)
          (- canvas-h margin (* 0.5 usable))
          (- canvas-h margin (* (/ usable span) (- y ymin)))))))

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
           (x0 (plot-scale-x (max xmin (min xmax 0.0))
                             xmin xmax margin w))
           (y0 (plot-scale-y (max ymin (min ymax 0.0))
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
    "Draw a single continuous segment provided as a list of (x . y) pairs,
     clipping lines to [YMIN, YMAX] and using draw-point for isolated points."
    (let* ((w (canvas-width))
           (h (canvas-height))
           (xspan (exact->inexact (- xmax xmin)))
           (yspan (exact->inexact (- ymax ymin)))
           (usable-w (exact->inexact (- w (* 2 margin))))
           (usable-h (exact->inexact (- h (* 2 margin))))
           (sx (if (= xspan 0.0) 0.0 (/ usable-w xspan)))
           (sy (if (= yspan 0.0) 0.0 (/ usable-h yspan)))
           (ox (if (= xspan 0.0)
                   (+ margin (* 0.5 usable-w))
                   (- margin (* sx xmin))))
           (oy (if (= yspan 0.0)
                   (- h margin (* 0.5 usable-h))
                   (+ (- h margin) (* sy ymin)))))
      (define px (lambda (x) (+ ox (* sx x))))
      (define py (lambda (y) (- oy (* sy y))))
      (define clip-x
        (lambda (xa ya xb yb yc)
          (+ xa (* (/ (- yc ya) (- yb ya)) (- xb xa)))))
      (define inside? (lambda (y) (and (<= ymin y) (<= y ymax))))
      ;; px0/py0 are the pixel coordinates of the current point when it
      ;; lies inside [ymin, ymax], so each point is scaled only once.
      (define draw-pairs
        (lambda (pts px0 py0)
          (if (null? (cdr pts))
              '()
              (let* ((p1 (car pts))
                     (p2 (cadr pts))
                     (x1 (car p1))
                     (y1 (cdr p1))
                     (x2 (car p2))
                     (y2 (cdr p2)))
                (cond
                  ((and (inside? y1) (inside? y2))
                   (let ((qx (px x2))
                         (qy (py y2))
                         (sx1 (or px0 (px x1)))
                         (sy1 (or py0 (py y1))))
                     (draw-line sx1 sy1 qx qy)
                     (draw-pairs (cdr pts) qx qy)))
                  ((or (and (< y1 ymin) (< y2 ymin))
                       (and (> y1 ymax) (> y2 ymax)))
                   (draw-pairs (cdr pts) #f #f))
                  (else
                   (let* ((cy1 (cond ((< y1 ymin) ymin) ((> y1 ymax) ymax) (else y1)))
                          (cy2 (cond ((< y2 ymin) ymin) ((> y2 ymax) ymax) (else y2)))
                          (cx1 (if (= cy1 y1) x1 (clip-x x1 y1 x2 y2 cy1)))
                          (cx2 (if (= cy2 y2) x2 (clip-x x2 y2 x1 y1 cy2))))
                     (draw-line (px cx1) (py cy1) (px cx2) (py cy2))
                     (draw-pairs (cdr pts) #f #f))))))))
      (cond
        ((null? segment) '())
        ((null? (cdr segment))
         (let ((x (car (car segment)))
               (y (cdr (car segment))))
           (if (inside? y)
               (draw-point (px x) (py y))
               '())))
        (else (draw-pairs segment #f #f))))))

(define plot-sort-numbers
  (lambda (xs)
    "Return a new list with the numbers of XS sorted in ascending order."
    (define merge
      (lambda (left right acc)
        (cond
          ((null? left) (append (reverse acc) right))
          ((null? right) (append (reverse acc) left))
          ((<= (car left) (car right))
           (merge (cdr left) right (cons (car left) acc)))
          (else
           (merge left (cdr right) (cons (car right) acc))))))
    (define take
      (lambda (lst k acc)
        (if (= k 0)
            (reverse acc)
            (take (cdr lst) (- k 1) (cons (car lst) acc)))))
    (let ((n (length xs)))
      (if (<= n 1)
          xs
          (let ((mid (quotient n 2)))
            (merge (plot-sort-numbers (take xs mid '()))
                   (plot-sort-numbers (list-tail xs mid))
                   '()))))))

(define plot-split-asymptotes
  (lambda (f segments core-span)
    "Split SEGMENTS wherever consecutive points jump across a vertical
     asymptote: when |y2 - y1| exceeds 0.25 * CORE-SPAN and the midpoint
     value f((x1 + x2) / 2) is non-finite or escapes [min(y1, y2), max(y1, y2)]."
    (let ((jump-threshold (* 0.25 core-span)))
      (define pole-between?
        (lambda (p1 p2)
          (let ((x1 (car p1))
                (y1 (cdr p1))
                (x2 (car p2))
                (y2 (cdr p2)))
            (and (> (abs (- y2 y1)) jump-threshold)
                 (let* ((ym (f (* 0.5 (+ x1 x2))))
                        (ylo (min y1 y2))
                        (yhi (max y1 y2))
                        (tol (* 1e-9 (- yhi ylo))))
                   (or (not (number? ym))
                       (not (finite? ym))
                       (< ym (- ylo tol))
                       (> ym (+ yhi tol))))))))
      (define close-current
        (lambda (acc current)
          (if (null? current)
              acc
              (cons (reverse current) acc))))
      (define split-one
        (lambda (pts current acc)
          (cond
            ((null? pts) (close-current acc current))
            ((null? current) (split-one (cdr pts) (list (car pts)) acc))
            ((pole-between? (car current) (car pts))
             (split-one (cdr pts)
                        (list (car pts))
                        (close-current acc current)))
            (else
             (split-one (cdr pts) (cons (car pts) current) acc)))))
      (let loop ((segs segments) (acc '()))
        (if (null? segs)
            (reverse acc)
            (loop (cdr segs) (split-one (car segs) '() acc)))))))

(define plot-function
  (lambda (f xmin xmax)
    "Plot the one-argument function F over [XMIN, XMAX].  Sampling points
     are connected within each continuous segment, split across poles and
     non-finite discontinuities, auto-scaled in Y (clamping asymptotic
     spikes when discontinuities are present), and redrawn on window resize."
    (define collect-ys
      (lambda (segments)
        (let loop-segs ((segs segments) (acc '()))
          (if (null? segs)
              (reverse acc)
              (let loop-pts ((pts (car segs)) (acc acc))
                (if (null? pts)
                    (loop-segs (cdr segs) acc)
                    (loop-pts (cdr pts) (cons (cdr (car pts)) acc))))))))
    (let* ((n-samples 401)
           (raw-segments (function-samples f xmin xmax n-samples))
           (ys (plot-sort-numbers (collect-ys raw-segments)))
           (xmin-view (exact->inexact xmin))
           (xmax-view (exact->inexact xmax))
           (margin 40))
      (if (null? ys)
          (set-redraw-hook!
            (lambda (w h)
              (clear-canvas)
              (plot-axes xmin-view xmax-view -1.0 1.0)))
          (let* ((m (length ys))
                 (ymin-raw (car ys))
                 (ymax-raw (list-ref ys (- m 1)))
                 (q10 (list-ref ys (quotient (* 10 (- m 1)) 100)))
                 (q90 (list-ref ys (quotient (* 90 (- m 1)) 100)))
                 (core-span (- q90 q10))
                 (segments (plot-split-asymptotes f raw-segments core-span))
                 (has-discontinuity (or (< m n-samples)
                                        (> (length segments) 1)))
                 (clamp? (and has-discontinuity (> core-span 0.0)))
                 (ymin-clamped (if clamp?
                                   (max ymin-raw (- q10 (* 1.5 core-span)))
                                   ymin-raw))
                 (ymax-clamped (if clamp?
                                   (min ymax-raw (+ q90 (* 1.5 core-span)))
                                   ymax-raw))
                 (flat? (= ymin-clamped ymax-clamped))
                 (ymin-view (if flat? (- ymin-clamped 1.0) ymin-clamped))
                 (ymax-view (if flat? (+ ymax-clamped 1.0) ymax-clamped)))
            (set-redraw-hook!
              (lambda (w h)
                (clear-canvas)
                (plot-axes xmin-view xmax-view ymin-view ymax-view)
                (set-color 0.9 0.3 0.2)
                (for-each
                  (lambda (seg)
                    (plot-segment seg xmin-view xmax-view ymin-view ymax-view margin))
                  segments))))))))
