import qbs 1.0

Application {
   property string rootDir: sourceDirectory + "/../"

   name: "thermo"
   files: [
      "main.cpp"           ,
      "MakeUnique.h"       ,
      "TJwt.cpp"           ,
      "TJwt.h"             ,
      "TJwt_p.h"           ,
      "TDriver.cpp"        ,
      "TDriver.h"          ,
      "TTempPoller.h"      ,
      "TTempPoller.cpp"    ,
      "TGCMqtt.cpp"        ,
      "TGCMqtt.h"          ,
   ]
   Depends { name: "cpp" }
   Depends { name: "Qt.core" }
   Depends { name: "Qt.network" }
   // cpp.compilerName: "clang++"
   // cpp.cxxStandardLibrary: "libstdc++"
   cpp.debugInformation: true
   cpp.cxxLanguageVersion: "c++17"
   //cpp.dynamicLibraries: [ "netfilter_queue", "pthread" ]
   //cpp.libraryPaths: ["/usr/lib/x86_64-linux-gnu/"]

   cpp.includePaths: [rootDir + "/../qtmqtt/include",
                      // "/usr/include"
   ]
   cpp.dynamicLibraries: [// "asan",
                          rootDir + "/../qtmqtt/lib/libQt5Mqtt.so",
                          "crypto",
                          "ssl",

   ]

   // clang asan
   // cpp.cxxFlags: ["-fsanitize=address", "-fno-omit-frame-pointer"]
   // cpp.cFlags: ["-fsanitize=address", "-fno-omit-frame-pointer"]


   Group {
      name: "The App itself"
      fileTagsFilter: "application"
      qbs.install: true
   }
}
