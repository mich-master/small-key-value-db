# small-key-value-db
Small Key/Value Database
_____
Небольшая key/value база данных для хранения строковых значений.  
Работает по протоколу HTTP, REST-подобный интерфейс.  
Использует IP-адрес, порт: 127.0.0.1:1080.  
В качестве клиента можно использовать любой HTTP-клиент.  
Примеры с использованием Curl.  

Добавить значение
```Shell
curl -X 'POST' -i -d '1_value' \http://127.0.0.1:1080/1111
curl -X 'POST' -i -d '2_value' \http://127.0.0.1:1080/2222
curl -X 'POST' -i -d '4_try' \http://127.0.0.1:1080/4444
```
Изменить значение
```Shell
curl -X 'PUT' -i -d '1_value_updated' \http://127.0.0.1:1080/1111
```

Удалить значение
```Shell
curl -X 'DELETE' -i \http://127.0.0.1:1080/2222
```

Получить значение
```Shell
curl -X 'GET' -i \http://127.0.0.1:1080/1111
```

Получать значения методом GET можно указанием ключа в строке браузера.  
