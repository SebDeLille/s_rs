;;; function-samples.scm -- Sample a one-variable Scheme function for
;;; plotting.
;;;
;;; This is written in plain Scheme (not a Rust primitive): it only
;;; relies on generic procedures already available in the interpreter
;;; (`do`, `apply`-free direct application, `exact?`/`inexact?`, and the
;;; `nan?`/`finite?` extensions from libsrs). It is loaded once by
;;; srsgtk at startup (see srsgtk/src/main.rs), before user startup
;;; scripts, so it is available to any .scm script drawing a graph.

(define function-samples
  (lambda (f xmin xmax n)
    "Sample F (a one-argument procedure) at N regularly spaced points
     between XMIN and XMAX (always coerced to Float, so a division by
     zero inside F, e.g. `tan` near an asymptote, produces +inf.0/nan.0
     instead of an exact-arithmetic error). Returns a list of segments,
     each segment being a list of (x . y) pairs in increasing x order.
     A sample is dropped, and the current segment closed, whenever F's
     result is not a number or is not finite (NaN or +/-infinity) --
     this avoids drawing a line across a discontinuity or asymptote.
     Note: an evaluation error raised by F itself (e.g. a wrong-type
     argument) is NOT caught -- the interpreter has no guard/catch
     mechanism -- and propagates out of function-samples."
    (define lo (exact->inexact xmin))
    (define hi (exact->inexact xmax))
    (define x-step (if (<= n 1) 0.0 (/ (- hi lo) (- n 1))))
    (define numeric?
      (lambda (v) (if (exact? v) #t (inexact? v))))
    (define valid-y?
      (lambda (y) (if (numeric? y) (finite? y) #f)))
    (define close-segment
      (lambda (segments current)
        (if (null? current)
            segments
            (cons (reverse current) segments))))
    (define step-state
      (lambda (state i)
        (let* ((segments (car state))
               (current (cdr state))
               (x (+ lo (* i x-step)))
               (y (f x)))
          (if (valid-y? y)
              (cons segments (cons (cons x y) current))
              (cons (close-segment segments current) '())))))
    (let ((final-state
           (do ((i 0 (+ i 1))
                (state (cons '() '()) (step-state state i)))
               ((= i n) state))))
      (reverse (close-segment (car final-state) (cdr final-state))))))
