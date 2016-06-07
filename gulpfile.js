'use strict';
 
var gulp = require('gulp');
var sass = require('gulp-sass');

gulp.task('sass', function () {
  return gulp.src('./src/sass/base.scss')
    .pipe(sass.sync({outputStyle: 'compressed'}).on('error', sass.logError))
    .pipe(gulp.dest('./static/css/'));
});
 
gulp.task('sass:watch', function () {
  gulp.watch('./src/sass/base.scss', ['sass']);
});

gulp.task('default',['sass:watch']);


