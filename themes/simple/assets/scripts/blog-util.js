// 辅助函数：计算文件的SHA256（示例，基于Web Crypto API）
async function sha256(file) {
    const arrayBuffer = await file.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
}

// 生成6位随机数字密码的函数
function generate_random_pwd(input_element_id) {
    // 1. 生成6位随机数（范围 100000 ~ 999999，确保是6位）
    let randomPwd = Math.floor(Math.random() * 900000) + 100000;
    // 2. 获取密码输入框元素
    let pwdInput = document.getElementById(input_element_id);
    // 3. 将随机数赋值给输入框（转字符串，避免数字格式问题）
    pwdInput.value = randomPwd.toString();
}