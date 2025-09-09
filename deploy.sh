# 1. 复制服务文件到系统目录
sudo cp jw-dioxus.service /etc/systemd/system/

# 2. 重新加载systemd配置
sudo systemctl daemon-reload

# 3. 启用开机自启动
sudo systemctl enable jw-dioxus.service

# 4. 立即启动服务
sudo systemctl start jw-dioxus.service
