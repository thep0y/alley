plugins {
    `kotlin-dsl`
}

gradlePlugin {
    plugins {
        create("pluginsForCoolKids") {
            id = "rust"
            implementationClass = "RustPlugin"
        }
    }
}

repositories {
    maven { url=uri ("https://mirrors.huaweicloud.com/repository/maven/")}
    maven { url=uri ("https://www.jitpack.io")}
    maven { url=uri ("https://maven.aliyun.com/repository/releases")}
    maven { url=uri ("https://maven.aliyun.com/repository/google")}
    maven { url=uri ("https://maven.aliyun.com/repository/central")}
    maven { url=uri ("https://maven.aliyun.com/repository/gradle-plugin")}
    maven { url=uri ("https://maven.aliyun.com/repository/public")}
    
    google()
    mavenCentral()
}

dependencies {
    compileOnly(gradleApi())
    implementation("com.android.tools.build:gradle:8.2.1")
}

