import qbs 1.0

Application {
   property string rootDir: sourceDirectory + "/../"

   name: "thermo"
   files: [
      "main.cpp"           ,
      "TMinMaxTracker.h"   ,
      "TMinMaxTracker.cpp" ,
      "TDriver.cpp"        ,
      "TDriver.h"          ,
      "TTempPoller.h"      ,
      "TTempPoller.cpp"    ,
      "TSmsSender.cpp"     ,
      "TSmsSender.h"       ,
      "TSmsCategoryIds.h"
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

   cpp.includePaths: [rootDir + "/../qtmqtt/include"]
   cpp.dynamicLibraries: [rootDir + "/../qtmqtt/lib/libQt5Mqtt.so"]


   Group {
      name: "The App itself"
      fileTagsFilter: "application"
      qbs.install: true
   }
}
