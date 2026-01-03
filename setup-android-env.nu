# Android 开发环境变量配置脚本
# 用法: source setup-android-env.nu

# 设置Java 17路径（根据实际安装路径调整）
$env.JAVA_HOME = "C:/Program Files/Eclipse Adoptium/jdk-17.0.17.10-hotspot/"

# 设置Android SDK路径
$env.ANDROID_HOME = "E:/android-sdk"

# 设置NDK路径
$env.NDK_HOME = "E:/android-sdk/ndk/29.0.14206865"

# 同时设置ANDROID_NDK_HOME（某些工具需要）
$env.ANDROID_NDK_HOME = "E:/android-sdk/ndk/29.0.14206865"

# 添加Java和Android工具到PATH（Java优先级最高）
$env.PATH = ($env.PATH | split row (char esep) 
    | prepend ($env.JAVA_HOME + "/bin")
    | append "E:/android-sdk/platform-tools" 
    | append "E:/android-sdk/cmdline-tools/latest/bin" 
    | str join (char esep))

# 显示配置信息
print "✅ Android开发环境变量已配置："
print $"  JAVA_HOME: ($env.JAVA_HOME)"
print $"  ANDROID_HOME: ($env.ANDROID_HOME)"
print $"  NDK_HOME: ($env.NDK_HOME)"
print ""

# 验证Java版本
print "验证Java安装："
if (($env.JAVA_HOME + "/bin/java.exe" | path exists)) {
    print "  ✅ Java可执行文件存在"
    # 显示Java版本
    try {
        let java_version = (^java -version | complete | get stderr | lines | first)
        print $"  版本: ($java_version)"
    } catch {
        print "  ⚠️  无法获取Java版本"
    }
} else {
    print "  ❌ Java未找到，请检查JAVA_HOME路径"
}

print ""
print "验证NDK安装："
if (("E:/android-sdk/ndk/29.0.14206865" | path exists)) {
    print "  ✅ NDK目录存在"
} else {
    print "  ❌ NDK目录不存在"
}

print ""
print "现在可以运行: cargo tauri android dev"
print "或构建APK: cargo tauri android build"
