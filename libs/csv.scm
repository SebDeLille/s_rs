;;; csv.scm -- Minimal CSV reading helpers for s_rs.
;;;
;;; All values stay as strings.  No quoting/escaping (RFC 4180) is handled
;;; in this first version: fields are simply split on the delimiter.

(define csv-parse-line
  (lambda (line delimiter)
    "Parse LINE into a list of field strings, splitting on DELIMITER."
    (let ((len (string-length line))
          (d (string-ref delimiter 0)))
      (do ((i 0 (+ i 1))
           (field '() (if (char=? (string-ref line i) d)
                          '()
                          (cons (string-ref line i) field)))
           (fields '() (if (char=? (string-ref line i) d)
                           (cons (list->string (reverse field)) fields)
                           fields)))
          ((= i len) (reverse (cons (list->string (reverse field)) fields)))
        'ok))))

(define csv-call-with-file
  (lambda (path delimiter callback)
    "Open PATH as a streaming CSV, parse the header once, then call CALLBACK
with (NEXT-ROW HEADER). NEXT-ROW returns the next parsed row as a list of
field strings, or '() at EOF. The input port is closed on normal return or
if CALLBACK raises an error."
    (let ((port (open-input-file path)))
      (define next-row
        (lambda ()
          (let ((line (read-line port)))
            (if (eof-object? line)
                '()
                (if (= (string-length line) 0)
                    (next-row)
                    (csv-parse-line line delimiter))))))
      (let ((header (next-row)))
        (dynamic-wind
          (lambda () 'ok)
          (lambda () (callback next-row header))
          (lambda () (close-input-port port)))))))

(define csv-for-each-row
  (lambda (path delimiter row-fn)
    "Iterate over the data rows of the CSV at PATH. For each row, call
ROW-FN with (HEADER ROW). The file is read lazily, one line at a time."
    (csv-call-with-file
      path
      delimiter
      (lambda (next-row header)
        (define first-row (next-row))
        (do ((row first-row (next-row)))
            ((null? row) 'done)
          (row-fn header row))))))

(define csv-read-file
  (lambda (path delimiter)
    "Read the CSV file at PATH and return (cons header-vector rows-vector)."
    (csv-call-with-file
      path
      delimiter
      (lambda (next-row header)
        (define collect
          (lambda (acc)
            (let ((row (next-row)))
              (if (null? row)
                  (reverse acc)
                  (collect (cons row acc))))))
        (cons (list->vector header)
              (list->vector (map list->vector (collect '()))))))))

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
