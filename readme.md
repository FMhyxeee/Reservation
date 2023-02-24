# Reservation

这是跟随陈天老师的视频，完整的开发一个Rust项目。

## 项目介绍
一个资源预定系统

## 初始化
```shell
pre-commit install

git remote add origin https://github.com/FMhyxeee/reservation.git
git branch -M master
git push -u origin master
```


## dump sql
```shell
pg_dump -s postgres://hyx:hyx@localhost:5432/reservation > reservation/fixtures/dump.sql
```


## insert sql
```sql
    insert into rsvp.reservations(user_id, resource_id, timespan, note) values ('hyx','room-421', '("2022-11-22","2022-11-2
 3")','你好啊，我来了');

    insert into rsvp.reservations(user_id, resource_id, timespan, note) values ('hyx','room-421', '("2022-11-22","2022-11-2
 3")','你好啊，我来了');

    select * from rsvp.query('hyx', 'room-421', '("2022-11-20","2022-12-21")','pending', 1, true, 2);


    select * from rsvp.filter('hyx', null,'pending', 1, true, 10);
```
