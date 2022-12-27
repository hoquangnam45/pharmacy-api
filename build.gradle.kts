plugins {
  java
  id("org.springframework.boot") version "3.0.1"
  id("io.spring.dependency-management") version "1.1.0"
  id("com.diffplug.spotless") version "6.12.0"
}

group = "com.hoquangnam45.pharmacy"
version = "0.0.1-SNAPSHOT"
java.sourceCompatibility = JavaVersion.VERSION_17

configurations {
  compileOnly {
    extendsFrom(configurations.annotationProcessor.get())
  }
}

repositories {
  mavenLocal()
  mavenCentral()
  maven { url = uri("https://artifactory-oss.prod.netflix.net/artifactory/maven-oss-candidates") }
}

extra["springCloudVersion"] = "2022.0.0"

dependencies {
  implementation("org.springframework.boot:spring-boot-starter-actuator")
  implementation("org.springframework.boot:spring-boot-starter-data-jpa")
  implementation("org.springframework.boot:spring-boot-starter-data-r2dbc")
  implementation("org.springframework.boot:spring-boot-starter-data-redis")
  implementation("org.springframework.boot:spring-boot-starter-data-redis-reactive")
  implementation("org.springframework.boot:spring-boot-starter-security")
  implementation("org.springframework.boot:spring-boot-starter-validation")
  implementation("org.springframework.boot:spring-boot-starter-web")
  implementation("org.springframework.boot:spring-boot-starter-webflux")
  implementation("org.springframework.cloud:spring-cloud-starter-netflix-eureka-client")
  implementation("org.flywaydb:flyway-core")
  compileOnly("org.projectlombok:lombok")
  developmentOnly("org.springframework.boot:spring-boot-devtools")
  runtimeOnly("org.postgresql:postgresql")
  runtimeOnly("org.postgresql:r2dbc-postgresql")
  runtimeOnly("com.h2database:h2")
  annotationProcessor("org.springframework.boot:spring-boot-configuration-processor")
  annotationProcessor("org.projectlombok:lombok")
  testImplementation("org.springframework.boot:spring-boot-starter-test")
  testImplementation("io.projectreactor:reactor-test")
  testImplementation("org.springframework.security:spring-security-test")
}

dependencyManagement {
  imports {
    mavenBom("org.springframework.cloud:spring-cloud-dependencies:${property("springCloudVersion")}")
  }
}

tasks.withType<Test> {
  useJUnitPlatform()
}

// Disable generation of plain jar file along with far jar of spring
tasks.getByName<Jar>("jar") {
  enabled = false
}

tasks.register("updateGitHooks", Copy::class) {
  from("./scripts/hooks/")
  into("./.git/hooks/")
  include("*")
  fileMode = 0b111111101
}
tasks.withType<JavaCompile> {
  dependsOn("updateGitHooks")
}

configure<com.diffplug.gradle.spotless.SpotlessExtension> {
  format("json") {
    target("*.json")
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    prettier().config(mapOf("parser" to "json"))
  }
  format("xml") {
    target("*.xml", "*.xsd")
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    eclipseWtp(com.diffplug.spotless.extra.wtp.EclipseWtpFormatterStep.XML).configFile("spotless.xml.prefs")
  }
  format("yaml") {
    target("*.yml", "*.yaml")
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    prettier().config(mapOf("parser" to "yaml"))
  }
  format("misc") {
    // define the files to apply `misc` to
    target("*.md", ".gitignore", ".dockerignore", "Dockerfile")
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    // define the steps to apply to those files
    trimTrailingWhitespace()
    indentWithSpaces(2) // or spaces. Takes an integer argument if you don't like 4
    endWithNewline()
  }
  kotlinGradle {
    target("*.kts")
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    ktlint().editorConfigOverride(mapOf("indent_size" to "2", "continuation_indent_size" to "2"))
    trimTrailingWhitespace()
    endWithNewline()
  }
  // don't need to set target, it is inferred from java
  java {
    targetExclude("build/", "target/", ".gradle/", "cache/", "bin/")
    toggleOffOn() // Allow to disable spotless for specific parts of code with spotless:off
    removeUnusedImports() // removes any unused imports
    // eclipse formatter
    eclipse().configFile("./eclipse-formatter.xml")
    // fix formatting of type annotations
    formatAnnotations()
  }
}
