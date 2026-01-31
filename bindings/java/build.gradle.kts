plugins {
    `java-library`
}

group = "tw.com.fugle"
version = "0.2.0"

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

repositories {
    mavenCentral()
}

dependencies {
    // JNA required for UniFFI-generated bindings
    implementation("net.java.dev.jna:jna:5.14.0")

    // Testing
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

tasks.test {
    useJUnitPlatform()
}

// Source sets - generated code lives in generated/ package
sourceSets {
    main {
        java {
            srcDirs("src/main/java")
        }
    }
}
