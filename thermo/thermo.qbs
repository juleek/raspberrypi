import qbs 1.0

Application {
   name: "thermo"
   files: [
      "main.cpp"           ,
      "ParseTemp.cpp"      ,
      "ParseTemp.h"        ,
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
   cpp.compilerName: "clang++"
   cpp.cxxStandardLibrary: "libstdc++"
   cpp.debugInformation: true
   cpp.cxxLanguageVersion: "c++14"
   //cpp.dynamicLibraries: [ "netfilter_queue", "pthread" ]
   //cpp.libraryPaths: ["/usr/lib/x86_64-linux-gnu/"]
}