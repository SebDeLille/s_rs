;;; csv.scm -- Minimal CSV reading helpers for s_rs.
;;;
;;; All values stay as strings.  No quoting/escaping (RFC 4180) is handled
;;; in this first version: fields are simply split on the delimiter.

(define csv-parse-line
  (lambda (line delimiter)
    "Parse LINE into a list of field strings, splitting on DELIMITER."
    (let ((len (string-length line))
          (d (string-ref delimiter 0)))
      (define csv-parse-line-loop
        (lambda (i field fields)
          (if (= i len)
              (reverse (cons (list->string (reverse field)) fields))
              (let ((c (string-ref line i)))
                (if (string=? (list->string (list c)) (list->string (list d)))
                    (csv-parse-line-loop
                     (+ i 1)
                     '()
                     (cons (list->string (reverse field)) fields))
                    (csv-parse-line-loop
                     (+ i 1)
                     (cons c field)
                     fields))))))
      (csv-parse-line-loop 0 '() '()))))

(define csv-read-file
  (lambda (path delimiter)
    "Read the CSV file at PATH and return (cons header-vector rows-vector)."
    (let ((port (open-input-file path)))
      (define read-lines
        (lambda (acc)
          (let ((line (read-line port)))
            (if (eof-object? line)
                (reverse acc)
                (if (= (string-length line) 0)
                    (read-lines acc)
                    (read-lines (cons line acc)))))))
      (let ((lines (read-lines '())))
        (if (null? lines)
            (cons (vector) (vector))
            (cons (list->vector (csv-parse-line (car lines) delimiter))
                  (list->vector
                   (map (lambda (line)
                          (list->vector (csv-parse-line line delimiter)))
                        (cdr lines)))))))))

(define csv-header
  (lambda (csv)
    "Return the header vector of a CSV value."
    (car csv)))

(define csv-rows
  (lambda (csv)
    "Return the rows vector of a CSV value."
    (cdr csv)))

(define csv-col-index
  (lambda (csv name)
    "Return the column index for NAME, or -1 if not found."
    (let ((header (csv-header csv))
          (len (vector-length (csv-header csv))))
      (define csv-col-index-loop
        (lambda (i)
          (if (= i len)
              -1
              (if (string=? name (vector-ref header i))
                  i
                  (csv-col-index-loop (+ i 1))))))
      (csv-col-index-loop 0))))

(define csv-ref
  (lambda (csv row-idx col-name)
    "Return the string value at ROW-IDX in column COL-NAME."
    (let ((col (csv-col-index csv col-name)))
      (if (= col -1)
          ""
          (vector-ref (vector-ref (csv-rows csv) row-idx) col)))))
