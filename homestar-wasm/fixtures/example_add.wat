(module
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param i32 i32) (result i32)))
  (type (;3;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;4;) (func (param i32 i32)))
  (type (;5;) (func (param i32)))
  (type (;6;) (func (result i32)))
  (type (;7;) (func (param i32 i32 i32)))
  (type (;8;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;9;) (func (param i32 i32 i32) (result i32)))
  (func (;0;) (type 0))
  (func (;1;) (type 0))
  (func (;2;) (type 1) (param i32) (result i32)
    call 5
    local.get 0
    i32.const 2
    i32.add
  )
  (func (;3;) (type 2) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call 15
    local.set 2
    local.get 2
    return
  )
  (func (;4;) (type 3) (param i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    local.get 2
    local.get 3
    call 16
    local.set 4
    local.get 4
    return
  )
  (func (;5;) (type 0)
    block ;; label = @1
      i32.const 0
      i32.load8_u offset=1048580
      br_if 0 (;@1;)
      call 0
      i32.const 0
      i32.const 1
      i32.store8 offset=1048580
    end
  )
  (func (;6;) (type 3) (param i32 i32 i32 i32) (result i32)
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          local.get 1
          br_if 0 (;@3;)
          local.get 3
          i32.eqz
          br_if 1 (;@2;)
          local.get 3
          local.get 2
          call 3
          local.tee 2
          br_if 1 (;@2;)
          br 2 (;@1;)
        end
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        call 4
        local.tee 2
        i32.eqz
        br_if 1 (;@1;)
      end
      local.get 2
      return
    end
    unreachable
    unreachable
  )
  (func (;7;) (type 2) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              local.get 1
              i32.const 9
              i32.lt_u
              br_if 0 (;@5;)
              i32.const 16
              i32.const 8
              call 17
              local.get 1
              i32.gt_u
              br_if 1 (;@4;)
              br 2 (;@3;)
            end
            local.get 0
            call 8
            local.set 2
            br 2 (;@2;)
          end
          i32.const 16
          i32.const 8
          call 17
          local.set 1
        end
        call 36
        local.tee 3
        i32.const 8
        call 17
        local.set 4
        i32.const 20
        i32.const 8
        call 17
        local.set 5
        i32.const 16
        i32.const 8
        call 17
        local.set 6
        i32.const 0
        local.set 2
        i32.const 0
        i32.const 16
        i32.const 8
        call 17
        i32.const 2
        i32.shl
        i32.sub
        local.tee 7
        local.get 3
        local.get 6
        local.get 4
        local.get 5
        i32.add
        i32.add
        i32.sub
        i32.const -65544
        i32.add
        i32.const -9
        i32.and
        i32.const -3
        i32.add
        local.tee 3
        local.get 7
        local.get 3
        i32.lt_u
        select
        local.get 1
        i32.sub
        local.get 0
        i32.le_u
        br_if 0 (;@2;)
        local.get 1
        i32.const 16
        local.get 0
        i32.const 4
        i32.add
        i32.const 16
        i32.const 8
        call 17
        i32.const -5
        i32.add
        local.get 0
        i32.gt_u
        select
        i32.const 8
        call 17
        local.tee 4
        i32.add
        i32.const 16
        i32.const 8
        call 17
        i32.add
        i32.const -4
        i32.add
        call 8
        local.tee 3
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        call 37
        local.set 0
        block ;; label = @3
          block ;; label = @4
            local.get 1
            i32.const -1
            i32.add
            local.tee 2
            local.get 3
            i32.and
            br_if 0 (;@4;)
            local.get 0
            local.set 1
            br 1 (;@3;)
          end
          local.get 2
          local.get 3
          i32.add
          i32.const 0
          local.get 1
          i32.sub
          i32.and
          call 37
          local.set 2
          i32.const 16
          i32.const 8
          call 17
          local.set 3
          local.get 0
          call 22
          local.get 2
          i32.const 0
          local.get 1
          local.get 2
          local.get 0
          i32.sub
          local.get 3
          i32.gt_u
          select
          i32.add
          local.tee 1
          local.get 0
          i32.sub
          local.tee 2
          i32.sub
          local.set 3
          block ;; label = @4
            local.get 0
            call 27
            br_if 0 (;@4;)
            local.get 1
            local.get 3
            call 28
            local.get 0
            local.get 2
            call 28
            local.get 0
            local.get 2
            call 9
            br 1 (;@3;)
          end
          local.get 0
          i32.load
          local.set 0
          local.get 1
          local.get 3
          i32.store offset=4
          local.get 1
          local.get 0
          local.get 2
          i32.add
          i32.store
        end
        local.get 1
        call 27
        br_if 1 (;@1;)
        local.get 1
        call 22
        local.tee 0
        i32.const 16
        i32.const 8
        call 17
        local.get 4
        i32.add
        i32.le_u
        br_if 1 (;@1;)
        local.get 1
        local.get 4
        call 33
        local.set 2
        local.get 1
        local.get 4
        call 28
        local.get 2
        local.get 0
        local.get 4
        i32.sub
        local.tee 0
        call 28
        local.get 2
        local.get 0
        call 9
        br 1 (;@1;)
      end
      local.get 2
      return
    end
    local.get 1
    call 35
    local.set 0
    local.get 1
    call 27
    drop
    local.get 0
  )
  (func (;8;) (type 1) (param i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              block ;; label = @6
                block ;; label = @7
                  local.get 0
                  i32.const 245
                  i32.lt_u
                  br_if 0 (;@7;)
                  call 36
                  local.tee 2
                  i32.const 8
                  call 17
                  local.set 3
                  i32.const 20
                  i32.const 8
                  call 17
                  local.set 4
                  i32.const 16
                  i32.const 8
                  call 17
                  local.set 5
                  i32.const 0
                  local.set 6
                  i32.const 0
                  i32.const 16
                  i32.const 8
                  call 17
                  i32.const 2
                  i32.shl
                  i32.sub
                  local.tee 7
                  local.get 2
                  local.get 5
                  local.get 3
                  local.get 4
                  i32.add
                  i32.add
                  i32.sub
                  i32.const -65544
                  i32.add
                  i32.const -9
                  i32.and
                  i32.const -3
                  i32.add
                  local.tee 2
                  local.get 7
                  local.get 2
                  i32.lt_u
                  select
                  local.get 0
                  i32.le_u
                  br_if 6 (;@1;)
                  local.get 0
                  i32.const 4
                  i32.add
                  i32.const 8
                  call 17
                  local.set 2
                  i32.const 0
                  i32.load offset=1048996
                  i32.eqz
                  br_if 5 (;@2;)
                  i32.const 0
                  local.set 8
                  block ;; label = @8
                    local.get 2
                    i32.const 256
                    i32.lt_u
                    br_if 0 (;@8;)
                    i32.const 31
                    local.set 8
                    local.get 2
                    i32.const 16777215
                    i32.gt_u
                    br_if 0 (;@8;)
                    local.get 2
                    i32.const 6
                    local.get 2
                    i32.const 8
                    i32.shr_u
                    i32.clz
                    local.tee 0
                    i32.sub
                    i32.shr_u
                    i32.const 1
                    i32.and
                    local.get 0
                    i32.const 1
                    i32.shl
                    i32.sub
                    i32.const 62
                    i32.add
                    local.set 8
                  end
                  i32.const 0
                  local.get 2
                  i32.sub
                  local.set 6
                  local.get 8
                  i32.const 2
                  i32.shl
                  i32.const 1048584
                  i32.add
                  i32.load
                  local.tee 3
                  br_if 1 (;@6;)
                  i32.const 0
                  local.set 0
                  i32.const 0
                  local.set 4
                  br 2 (;@5;)
                end
                i32.const 16
                local.get 0
                i32.const 4
                i32.add
                i32.const 16
                i32.const 8
                call 17
                i32.const -5
                i32.add
                local.get 0
                i32.gt_u
                select
                i32.const 8
                call 17
                local.set 2
                block ;; label = @7
                  block ;; label = @8
                    block ;; label = @9
                      block ;; label = @10
                        block ;; label = @11
                          block ;; label = @12
                            block ;; label = @13
                              i32.const 0
                              i32.load offset=1048992
                              local.tee 4
                              local.get 2
                              i32.const 3
                              i32.shr_u
                              local.tee 6
                              i32.shr_u
                              local.tee 0
                              i32.const 3
                              i32.and
                              br_if 0 (;@13;)
                              local.get 2
                              i32.const 0
                              i32.load offset=1049000
                              i32.le_u
                              br_if 11 (;@2;)
                              local.get 0
                              br_if 1 (;@12;)
                              i32.const 0
                              i32.load offset=1048996
                              local.tee 0
                              i32.eqz
                              br_if 11 (;@2;)
                              local.get 0
                              call 19
                              i32.ctz
                              i32.const 2
                              i32.shl
                              i32.const 1048584
                              i32.add
                              i32.load
                              local.tee 3
                              call 39
                              call 22
                              local.get 2
                              i32.sub
                              local.set 6
                              block ;; label = @14
                                local.get 3
                                call 38
                                local.tee 0
                                i32.eqz
                                br_if 0 (;@14;)
                                loop ;; label = @15
                                  local.get 0
                                  call 39
                                  call 22
                                  local.get 2
                                  i32.sub
                                  local.tee 4
                                  local.get 6
                                  local.get 4
                                  local.get 6
                                  i32.lt_u
                                  local.tee 4
                                  select
                                  local.set 6
                                  local.get 0
                                  local.get 3
                                  local.get 4
                                  select
                                  local.set 3
                                  local.get 0
                                  call 38
                                  local.tee 0
                                  br_if 0 (;@15;)
                                end
                              end
                              local.get 3
                              call 39
                              local.tee 0
                              local.get 2
                              call 33
                              local.set 4
                              local.get 3
                              call 10
                              local.get 6
                              i32.const 16
                              i32.const 8
                              call 17
                              i32.lt_u
                              br_if 5 (;@8;)
                              local.get 4
                              call 39
                              local.set 4
                              local.get 0
                              local.get 2
                              call 30
                              local.get 4
                              local.get 6
                              call 31
                              i32.const 0
                              i32.load offset=1049000
                              local.tee 7
                              i32.eqz
                              br_if 4 (;@9;)
                              local.get 7
                              i32.const -8
                              i32.and
                              i32.const 1048728
                              i32.add
                              local.set 5
                              i32.const 0
                              i32.load offset=1049008
                              local.set 3
                              i32.const 0
                              i32.load offset=1048992
                              local.tee 8
                              i32.const 1
                              local.get 7
                              i32.const 3
                              i32.shr_u
                              i32.shl
                              local.tee 7
                              i32.and
                              i32.eqz
                              br_if 2 (;@11;)
                              local.get 5
                              i32.load offset=8
                              local.set 7
                              br 3 (;@10;)
                            end
                            block ;; label = @13
                              block ;; label = @14
                                local.get 0
                                i32.const -1
                                i32.xor
                                i32.const 1
                                i32.and
                                local.get 6
                                i32.add
                                local.tee 2
                                i32.const 3
                                i32.shl
                                local.tee 3
                                i32.const 1048736
                                i32.add
                                i32.load
                                local.tee 0
                                i32.const 8
                                i32.add
                                i32.load
                                local.tee 6
                                local.get 3
                                i32.const 1048728
                                i32.add
                                local.tee 3
                                i32.eq
                                br_if 0 (;@14;)
                                local.get 6
                                local.get 3
                                i32.store offset=12
                                local.get 3
                                local.get 6
                                i32.store offset=8
                                br 1 (;@13;)
                              end
                              i32.const 0
                              local.get 4
                              i32.const -2
                              local.get 2
                              i32.rotl
                              i32.and
                              i32.store offset=1048992
                            end
                            local.get 0
                            local.get 2
                            i32.const 3
                            i32.shl
                            call 29
                            local.get 0
                            call 35
                            local.set 6
                            br 11 (;@1;)
                          end
                          block ;; label = @12
                            block ;; label = @13
                              i32.const 1
                              local.get 6
                              i32.const 31
                              i32.and
                              local.tee 6
                              i32.shl
                              call 18
                              local.get 0
                              local.get 6
                              i32.shl
                              i32.and
                              call 19
                              i32.ctz
                              local.tee 6
                              i32.const 3
                              i32.shl
                              local.tee 4
                              i32.const 1048736
                              i32.add
                              i32.load
                              local.tee 0
                              i32.const 8
                              i32.add
                              i32.load
                              local.tee 3
                              local.get 4
                              i32.const 1048728
                              i32.add
                              local.tee 4
                              i32.eq
                              br_if 0 (;@13;)
                              local.get 3
                              local.get 4
                              i32.store offset=12
                              local.get 4
                              local.get 3
                              i32.store offset=8
                              br 1 (;@12;)
                            end
                            i32.const 0
                            i32.const 0
                            i32.load offset=1048992
                            i32.const -2
                            local.get 6
                            i32.rotl
                            i32.and
                            i32.store offset=1048992
                          end
                          local.get 0
                          local.get 2
                          call 30
                          local.get 0
                          local.get 2
                          call 33
                          local.tee 4
                          local.get 6
                          i32.const 3
                          i32.shl
                          local.get 2
                          i32.sub
                          local.tee 5
                          call 31
                          block ;; label = @12
                            i32.const 0
                            i32.load offset=1049000
                            local.tee 3
                            i32.eqz
                            br_if 0 (;@12;)
                            local.get 3
                            i32.const -8
                            i32.and
                            i32.const 1048728
                            i32.add
                            local.set 6
                            i32.const 0
                            i32.load offset=1049008
                            local.set 2
                            block ;; label = @13
                              block ;; label = @14
                                i32.const 0
                                i32.load offset=1048992
                                local.tee 7
                                i32.const 1
                                local.get 3
                                i32.const 3
                                i32.shr_u
                                i32.shl
                                local.tee 3
                                i32.and
                                i32.eqz
                                br_if 0 (;@14;)
                                local.get 6
                                i32.load offset=8
                                local.set 3
                                br 1 (;@13;)
                              end
                              i32.const 0
                              local.get 7
                              local.get 3
                              i32.or
                              i32.store offset=1048992
                              local.get 6
                              local.set 3
                            end
                            local.get 6
                            local.get 2
                            i32.store offset=8
                            local.get 3
                            local.get 2
                            i32.store offset=12
                            local.get 2
                            local.get 6
                            i32.store offset=12
                            local.get 2
                            local.get 3
                            i32.store offset=8
                          end
                          i32.const 0
                          local.get 4
                          i32.store offset=1049008
                          i32.const 0
                          local.get 5
                          i32.store offset=1049000
                          local.get 0
                          call 35
                          local.set 6
                          br 10 (;@1;)
                        end
                        i32.const 0
                        local.get 8
                        local.get 7
                        i32.or
                        i32.store offset=1048992
                        local.get 5
                        local.set 7
                      end
                      local.get 5
                      local.get 3
                      i32.store offset=8
                      local.get 7
                      local.get 3
                      i32.store offset=12
                      local.get 3
                      local.get 5
                      i32.store offset=12
                      local.get 3
                      local.get 7
                      i32.store offset=8
                    end
                    i32.const 0
                    local.get 4
                    i32.store offset=1049008
                    i32.const 0
                    local.get 6
                    i32.store offset=1049000
                    br 1 (;@7;)
                  end
                  local.get 0
                  local.get 6
                  local.get 2
                  i32.add
                  call 29
                end
                local.get 0
                call 35
                local.tee 6
                br_if 5 (;@1;)
                br 4 (;@2;)
              end
              local.get 2
              local.get 8
              call 20
              i32.shl
              local.set 5
              i32.const 0
              local.set 0
              i32.const 0
              local.set 4
              loop ;; label = @6
                block ;; label = @7
                  local.get 3
                  call 39
                  call 22
                  local.tee 7
                  local.get 2
                  i32.lt_u
                  br_if 0 (;@7;)
                  local.get 7
                  local.get 2
                  i32.sub
                  local.tee 7
                  local.get 6
                  i32.ge_u
                  br_if 0 (;@7;)
                  local.get 7
                  local.set 6
                  local.get 3
                  local.set 4
                  local.get 7
                  br_if 0 (;@7;)
                  i32.const 0
                  local.set 6
                  local.get 3
                  local.set 4
                  local.get 3
                  local.set 0
                  br 3 (;@4;)
                end
                local.get 3
                i32.const 20
                i32.add
                i32.load
                local.tee 7
                local.get 0
                local.get 7
                local.get 3
                local.get 5
                i32.const 29
                i32.shr_u
                i32.const 4
                i32.and
                i32.add
                i32.const 16
                i32.add
                i32.load
                local.tee 3
                i32.ne
                select
                local.get 0
                local.get 7
                select
                local.set 0
                local.get 5
                i32.const 1
                i32.shl
                local.set 5
                local.get 3
                br_if 0 (;@6;)
              end
            end
            block ;; label = @5
              local.get 0
              local.get 4
              i32.or
              br_if 0 (;@5;)
              i32.const 0
              local.set 4
              i32.const 1
              local.get 8
              i32.shl
              call 18
              i32.const 0
              i32.load offset=1048996
              i32.and
              local.tee 0
              i32.eqz
              br_if 3 (;@2;)
              local.get 0
              call 19
              i32.ctz
              i32.const 2
              i32.shl
              i32.const 1048584
              i32.add
              i32.load
              local.set 0
            end
            local.get 0
            i32.eqz
            br_if 1 (;@3;)
          end
          loop ;; label = @4
            local.get 0
            local.get 4
            local.get 0
            call 39
            call 22
            local.tee 3
            local.get 2
            i32.ge_u
            local.get 3
            local.get 2
            i32.sub
            local.tee 3
            local.get 6
            i32.lt_u
            i32.and
            local.tee 5
            select
            local.set 4
            local.get 3
            local.get 6
            local.get 5
            select
            local.set 6
            local.get 0
            call 38
            local.tee 0
            br_if 0 (;@4;)
          end
        end
        local.get 4
        i32.eqz
        br_if 0 (;@2;)
        block ;; label = @3
          i32.const 0
          i32.load offset=1049000
          local.tee 0
          local.get 2
          i32.lt_u
          br_if 0 (;@3;)
          local.get 6
          local.get 0
          local.get 2
          i32.sub
          i32.ge_u
          br_if 1 (;@2;)
        end
        local.get 4
        call 39
        local.tee 0
        local.get 2
        call 33
        local.set 3
        local.get 4
        call 10
        block ;; label = @3
          block ;; label = @4
            local.get 6
            i32.const 16
            i32.const 8
            call 17
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            local.get 2
            call 30
            local.get 3
            local.get 6
            call 31
            block ;; label = @5
              local.get 6
              i32.const 256
              i32.lt_u
              br_if 0 (;@5;)
              local.get 3
              local.get 6
              call 11
              br 2 (;@3;)
            end
            local.get 6
            i32.const -8
            i32.and
            i32.const 1048728
            i32.add
            local.set 4
            block ;; label = @5
              block ;; label = @6
                i32.const 0
                i32.load offset=1048992
                local.tee 5
                i32.const 1
                local.get 6
                i32.const 3
                i32.shr_u
                i32.shl
                local.tee 6
                i32.and
                i32.eqz
                br_if 0 (;@6;)
                local.get 4
                i32.load offset=8
                local.set 6
                br 1 (;@5;)
              end
              i32.const 0
              local.get 5
              local.get 6
              i32.or
              i32.store offset=1048992
              local.get 4
              local.set 6
            end
            local.get 4
            local.get 3
            i32.store offset=8
            local.get 6
            local.get 3
            i32.store offset=12
            local.get 3
            local.get 4
            i32.store offset=12
            local.get 3
            local.get 6
            i32.store offset=8
            br 1 (;@3;)
          end
          local.get 0
          local.get 6
          local.get 2
          i32.add
          call 29
        end
        local.get 0
        call 35
        local.tee 6
        br_if 1 (;@1;)
      end
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    block ;; label = @9
                      i32.const 0
                      i32.load offset=1049000
                      local.tee 6
                      local.get 2
                      i32.ge_u
                      br_if 0 (;@9;)
                      i32.const 0
                      i32.load offset=1049004
                      local.tee 0
                      local.get 2
                      i32.gt_u
                      br_if 2 (;@7;)
                      local.get 1
                      i32.const 1048584
                      local.get 2
                      call 36
                      local.tee 0
                      i32.sub
                      local.get 0
                      i32.const 8
                      call 17
                      i32.add
                      i32.const 20
                      i32.const 8
                      call 17
                      i32.add
                      i32.const 16
                      i32.const 8
                      call 17
                      i32.add
                      i32.const 8
                      i32.add
                      i32.const 65536
                      call 17
                      call 46
                      local.get 1
                      i32.load
                      local.tee 6
                      br_if 1 (;@8;)
                      i32.const 0
                      local.set 6
                      br 8 (;@1;)
                    end
                    i32.const 0
                    i32.load offset=1049008
                    local.set 0
                    block ;; label = @9
                      local.get 6
                      local.get 2
                      i32.sub
                      local.tee 6
                      i32.const 16
                      i32.const 8
                      call 17
                      i32.ge_u
                      br_if 0 (;@9;)
                      i32.const 0
                      i32.const 0
                      i32.store offset=1049008
                      i32.const 0
                      i32.load offset=1049000
                      local.set 2
                      i32.const 0
                      i32.const 0
                      i32.store offset=1049000
                      local.get 0
                      local.get 2
                      call 29
                      local.get 0
                      call 35
                      local.set 6
                      br 8 (;@1;)
                    end
                    local.get 0
                    local.get 2
                    call 33
                    local.set 3
                    i32.const 0
                    local.get 6
                    i32.store offset=1049000
                    i32.const 0
                    local.get 3
                    i32.store offset=1049008
                    local.get 3
                    local.get 6
                    call 31
                    local.get 0
                    local.get 2
                    call 30
                    local.get 0
                    call 35
                    local.set 6
                    br 7 (;@1;)
                  end
                  local.get 1
                  i32.load offset=8
                  local.set 8
                  i32.const 0
                  i32.const 0
                  i32.load offset=1049016
                  local.get 1
                  i32.load offset=4
                  local.tee 5
                  i32.add
                  local.tee 0
                  i32.store offset=1049016
                  i32.const 0
                  i32.const 0
                  i32.load offset=1049020
                  local.tee 3
                  local.get 0
                  local.get 3
                  local.get 0
                  i32.gt_u
                  select
                  i32.store offset=1049020
                  block ;; label = @8
                    block ;; label = @9
                      block ;; label = @10
                        block ;; label = @11
                          i32.const 0
                          i32.load offset=1049012
                          i32.eqz
                          br_if 0 (;@11;)
                          i32.const 1048712
                          local.set 0
                          loop ;; label = @12
                            local.get 6
                            local.get 0
                            call 45
                            i32.eq
                            br_if 2 (;@10;)
                            local.get 0
                            i32.load offset=8
                            local.tee 0
                            br_if 0 (;@12;)
                            br 3 (;@9;)
                          end
                        end
                        i32.const 0
                        i32.load offset=1049028
                        local.tee 0
                        i32.eqz
                        br_if 5 (;@5;)
                        local.get 6
                        local.get 0
                        i32.lt_u
                        br_if 5 (;@5;)
                        br 7 (;@3;)
                      end
                      local.get 0
                      call 42
                      br_if 0 (;@9;)
                      local.get 0
                      call 43
                      local.get 8
                      i32.ne
                      br_if 0 (;@9;)
                      local.get 0
                      i32.const 0
                      i32.load offset=1049012
                      call 44
                      br_if 1 (;@8;)
                    end
                    i32.const 0
                    i32.const 0
                    i32.load offset=1049028
                    local.tee 0
                    local.get 6
                    local.get 6
                    local.get 0
                    i32.gt_u
                    select
                    i32.store offset=1049028
                    local.get 6
                    local.get 5
                    i32.add
                    local.set 3
                    i32.const 1048712
                    local.set 0
                    block ;; label = @9
                      block ;; label = @10
                        block ;; label = @11
                          loop ;; label = @12
                            local.get 0
                            i32.load
                            local.get 3
                            i32.eq
                            br_if 1 (;@11;)
                            local.get 0
                            i32.load offset=8
                            local.tee 0
                            br_if 0 (;@12;)
                            br 2 (;@10;)
                          end
                        end
                        local.get 0
                        call 42
                        br_if 0 (;@10;)
                        local.get 0
                        call 43
                        local.get 8
                        i32.eq
                        br_if 1 (;@9;)
                      end
                      i32.const 0
                      i32.load offset=1049012
                      local.set 3
                      i32.const 1048712
                      local.set 0
                      block ;; label = @10
                        loop ;; label = @11
                          block ;; label = @12
                            local.get 0
                            i32.load
                            local.get 3
                            i32.gt_u
                            br_if 0 (;@12;)
                            local.get 0
                            call 45
                            local.get 3
                            i32.gt_u
                            br_if 2 (;@10;)
                          end
                          local.get 0
                          i32.load offset=8
                          local.tee 0
                          br_if 0 (;@11;)
                        end
                        i32.const 0
                        local.set 0
                      end
                      local.get 0
                      call 45
                      local.tee 4
                      i32.const 20
                      i32.const 8
                      call 17
                      local.tee 9
                      i32.sub
                      i32.const -23
                      i32.add
                      local.set 0
                      local.get 3
                      local.get 0
                      local.get 0
                      call 35
                      local.tee 7
                      i32.const 8
                      call 17
                      local.get 7
                      i32.sub
                      i32.add
                      local.tee 0
                      local.get 0
                      local.get 3
                      i32.const 16
                      i32.const 8
                      call 17
                      i32.add
                      i32.lt_u
                      select
                      local.tee 7
                      call 35
                      local.set 10
                      local.get 7
                      local.get 9
                      call 33
                      local.set 0
                      call 36
                      local.tee 11
                      i32.const 8
                      call 17
                      local.set 12
                      i32.const 20
                      i32.const 8
                      call 17
                      local.set 13
                      i32.const 16
                      i32.const 8
                      call 17
                      local.set 14
                      i32.const 0
                      local.get 6
                      local.get 6
                      call 35
                      local.tee 15
                      i32.const 8
                      call 17
                      local.get 15
                      i32.sub
                      local.tee 16
                      call 33
                      local.tee 15
                      i32.store offset=1049012
                      i32.const 0
                      local.get 11
                      local.get 5
                      i32.add
                      local.get 14
                      local.get 12
                      local.get 13
                      i32.add
                      i32.add
                      local.get 16
                      i32.add
                      i32.sub
                      local.tee 11
                      i32.store offset=1049004
                      local.get 15
                      local.get 11
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      call 36
                      local.tee 12
                      i32.const 8
                      call 17
                      local.set 13
                      i32.const 20
                      i32.const 8
                      call 17
                      local.set 14
                      i32.const 16
                      i32.const 8
                      call 17
                      local.set 16
                      local.get 15
                      local.get 11
                      call 33
                      local.get 16
                      local.get 14
                      local.get 13
                      local.get 12
                      i32.sub
                      i32.add
                      i32.add
                      i32.store offset=4
                      i32.const 0
                      i32.const 2097152
                      i32.store offset=1049024
                      local.get 7
                      local.get 9
                      call 30
                      i32.const 0
                      i64.load offset=1048712 align=4
                      local.set 17
                      local.get 10
                      i32.const 8
                      i32.add
                      i32.const 0
                      i64.load offset=1048720 align=4
                      i64.store align=4
                      local.get 10
                      local.get 17
                      i64.store align=4
                      i32.const 0
                      local.get 8
                      i32.store offset=1048724
                      i32.const 0
                      local.get 5
                      i32.store offset=1048716
                      i32.const 0
                      local.get 6
                      i32.store offset=1048712
                      i32.const 0
                      local.get 10
                      i32.store offset=1048720
                      loop ;; label = @10
                        local.get 0
                        i32.const 4
                        call 33
                        local.set 6
                        local.get 0
                        call 21
                        i32.store offset=4
                        local.get 6
                        local.set 0
                        local.get 6
                        i32.const 4
                        i32.add
                        local.get 4
                        i32.lt_u
                        br_if 0 (;@10;)
                      end
                      local.get 7
                      local.get 3
                      i32.eq
                      br_if 7 (;@2;)
                      local.get 7
                      local.get 3
                      i32.sub
                      local.set 0
                      local.get 3
                      local.get 0
                      local.get 3
                      local.get 0
                      call 33
                      call 32
                      block ;; label = @10
                        local.get 0
                        i32.const 256
                        i32.lt_u
                        br_if 0 (;@10;)
                        local.get 3
                        local.get 0
                        call 11
                        br 8 (;@2;)
                      end
                      local.get 0
                      i32.const -8
                      i32.and
                      i32.const 1048728
                      i32.add
                      local.set 6
                      block ;; label = @10
                        block ;; label = @11
                          i32.const 0
                          i32.load offset=1048992
                          local.tee 4
                          i32.const 1
                          local.get 0
                          i32.const 3
                          i32.shr_u
                          i32.shl
                          local.tee 0
                          i32.and
                          i32.eqz
                          br_if 0 (;@11;)
                          local.get 6
                          i32.load offset=8
                          local.set 0
                          br 1 (;@10;)
                        end
                        i32.const 0
                        local.get 4
                        local.get 0
                        i32.or
                        i32.store offset=1048992
                        local.get 6
                        local.set 0
                      end
                      local.get 6
                      local.get 3
                      i32.store offset=8
                      local.get 0
                      local.get 3
                      i32.store offset=12
                      local.get 3
                      local.get 6
                      i32.store offset=12
                      local.get 3
                      local.get 0
                      i32.store offset=8
                      br 7 (;@2;)
                    end
                    local.get 0
                    i32.load
                    local.set 4
                    local.get 0
                    local.get 6
                    i32.store
                    local.get 0
                    local.get 0
                    i32.load offset=4
                    local.get 5
                    i32.add
                    i32.store offset=4
                    local.get 6
                    call 35
                    local.tee 0
                    i32.const 8
                    call 17
                    local.set 3
                    local.get 4
                    call 35
                    local.tee 5
                    i32.const 8
                    call 17
                    local.set 7
                    local.get 6
                    local.get 3
                    local.get 0
                    i32.sub
                    i32.add
                    local.tee 6
                    local.get 2
                    call 33
                    local.set 3
                    local.get 6
                    local.get 2
                    call 30
                    local.get 4
                    local.get 7
                    local.get 5
                    i32.sub
                    i32.add
                    local.tee 0
                    local.get 2
                    local.get 6
                    i32.add
                    i32.sub
                    local.set 2
                    block ;; label = @9
                      local.get 0
                      i32.const 0
                      i32.load offset=1049012
                      i32.eq
                      br_if 0 (;@9;)
                      local.get 0
                      i32.const 0
                      i32.load offset=1049008
                      i32.eq
                      br_if 3 (;@6;)
                      local.get 0
                      call 26
                      br_if 5 (;@4;)
                      block ;; label = @10
                        block ;; label = @11
                          local.get 0
                          call 22
                          local.tee 4
                          i32.const 256
                          i32.lt_u
                          br_if 0 (;@11;)
                          local.get 0
                          call 10
                          br 1 (;@10;)
                        end
                        block ;; label = @11
                          local.get 0
                          i32.const 12
                          i32.add
                          i32.load
                          local.tee 5
                          local.get 0
                          i32.const 8
                          i32.add
                          i32.load
                          local.tee 7
                          i32.eq
                          br_if 0 (;@11;)
                          local.get 7
                          local.get 5
                          i32.store offset=12
                          local.get 5
                          local.get 7
                          i32.store offset=8
                          br 1 (;@10;)
                        end
                        i32.const 0
                        i32.const 0
                        i32.load offset=1048992
                        i32.const -2
                        local.get 4
                        i32.const 3
                        i32.shr_u
                        i32.rotl
                        i32.and
                        i32.store offset=1048992
                      end
                      local.get 4
                      local.get 2
                      i32.add
                      local.set 2
                      local.get 0
                      local.get 4
                      call 33
                      local.set 0
                      br 5 (;@4;)
                    end
                    i32.const 0
                    local.get 3
                    i32.store offset=1049012
                    i32.const 0
                    i32.const 0
                    i32.load offset=1049004
                    local.get 2
                    i32.add
                    local.tee 0
                    i32.store offset=1049004
                    local.get 3
                    local.get 0
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    local.get 6
                    call 35
                    local.set 6
                    br 7 (;@1;)
                  end
                  local.get 0
                  local.get 0
                  i32.load offset=4
                  local.get 5
                  i32.add
                  i32.store offset=4
                  i32.const 0
                  i32.load offset=1049012
                  i32.const 0
                  i32.load offset=1049004
                  local.get 5
                  i32.add
                  call 14
                  br 5 (;@2;)
                end
                i32.const 0
                local.get 0
                local.get 2
                i32.sub
                local.tee 6
                i32.store offset=1049004
                i32.const 0
                i32.const 0
                i32.load offset=1049012
                local.tee 0
                local.get 2
                call 33
                local.tee 3
                i32.store offset=1049012
                local.get 3
                local.get 6
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                local.get 2
                call 30
                local.get 0
                call 35
                local.set 6
                br 5 (;@1;)
              end
              i32.const 0
              local.get 3
              i32.store offset=1049008
              i32.const 0
              i32.const 0
              i32.load offset=1049000
              local.get 2
              i32.add
              local.tee 0
              i32.store offset=1049000
              local.get 3
              local.get 0
              call 31
              local.get 6
              call 35
              local.set 6
              br 4 (;@1;)
            end
            i32.const 0
            local.get 6
            i32.store offset=1049028
            br 1 (;@3;)
          end
          local.get 3
          local.get 2
          local.get 0
          call 32
          block ;; label = @4
            local.get 2
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 3
            local.get 2
            call 11
            local.get 6
            call 35
            local.set 6
            br 3 (;@1;)
          end
          local.get 2
          i32.const -8
          i32.and
          i32.const 1048728
          i32.add
          local.set 0
          block ;; label = @4
            block ;; label = @5
              i32.const 0
              i32.load offset=1048992
              local.tee 4
              i32.const 1
              local.get 2
              i32.const 3
              i32.shr_u
              i32.shl
              local.tee 2
              i32.and
              i32.eqz
              br_if 0 (;@5;)
              local.get 0
              i32.load offset=8
              local.set 2
              br 1 (;@4;)
            end
            i32.const 0
            local.get 4
            local.get 2
            i32.or
            i32.store offset=1048992
            local.get 0
            local.set 2
          end
          local.get 0
          local.get 3
          i32.store offset=8
          local.get 2
          local.get 3
          i32.store offset=12
          local.get 3
          local.get 0
          i32.store offset=12
          local.get 3
          local.get 2
          i32.store offset=8
          local.get 6
          call 35
          local.set 6
          br 2 (;@1;)
        end
        i32.const 0
        i32.const 4095
        i32.store offset=1049032
        i32.const 0
        local.get 8
        i32.store offset=1048724
        i32.const 0
        local.get 5
        i32.store offset=1048716
        i32.const 0
        local.get 6
        i32.store offset=1048712
        i32.const 0
        i32.const 1048728
        i32.store offset=1048740
        i32.const 0
        i32.const 1048736
        i32.store offset=1048748
        i32.const 0
        i32.const 1048728
        i32.store offset=1048736
        i32.const 0
        i32.const 1048744
        i32.store offset=1048756
        i32.const 0
        i32.const 1048736
        i32.store offset=1048744
        i32.const 0
        i32.const 1048752
        i32.store offset=1048764
        i32.const 0
        i32.const 1048744
        i32.store offset=1048752
        i32.const 0
        i32.const 1048760
        i32.store offset=1048772
        i32.const 0
        i32.const 1048752
        i32.store offset=1048760
        i32.const 0
        i32.const 1048768
        i32.store offset=1048780
        i32.const 0
        i32.const 1048760
        i32.store offset=1048768
        i32.const 0
        i32.const 1048776
        i32.store offset=1048788
        i32.const 0
        i32.const 1048768
        i32.store offset=1048776
        i32.const 0
        i32.const 1048784
        i32.store offset=1048796
        i32.const 0
        i32.const 1048776
        i32.store offset=1048784
        i32.const 0
        i32.const 1048792
        i32.store offset=1048804
        i32.const 0
        i32.const 1048784
        i32.store offset=1048792
        i32.const 0
        i32.const 1048792
        i32.store offset=1048800
        i32.const 0
        i32.const 1048800
        i32.store offset=1048812
        i32.const 0
        i32.const 1048800
        i32.store offset=1048808
        i32.const 0
        i32.const 1048808
        i32.store offset=1048820
        i32.const 0
        i32.const 1048808
        i32.store offset=1048816
        i32.const 0
        i32.const 1048816
        i32.store offset=1048828
        i32.const 0
        i32.const 1048816
        i32.store offset=1048824
        i32.const 0
        i32.const 1048824
        i32.store offset=1048836
        i32.const 0
        i32.const 1048824
        i32.store offset=1048832
        i32.const 0
        i32.const 1048832
        i32.store offset=1048844
        i32.const 0
        i32.const 1048832
        i32.store offset=1048840
        i32.const 0
        i32.const 1048840
        i32.store offset=1048852
        i32.const 0
        i32.const 1048840
        i32.store offset=1048848
        i32.const 0
        i32.const 1048848
        i32.store offset=1048860
        i32.const 0
        i32.const 1048848
        i32.store offset=1048856
        i32.const 0
        i32.const 1048856
        i32.store offset=1048868
        i32.const 0
        i32.const 1048864
        i32.store offset=1048876
        i32.const 0
        i32.const 1048856
        i32.store offset=1048864
        i32.const 0
        i32.const 1048872
        i32.store offset=1048884
        i32.const 0
        i32.const 1048864
        i32.store offset=1048872
        i32.const 0
        i32.const 1048880
        i32.store offset=1048892
        i32.const 0
        i32.const 1048872
        i32.store offset=1048880
        i32.const 0
        i32.const 1048888
        i32.store offset=1048900
        i32.const 0
        i32.const 1048880
        i32.store offset=1048888
        i32.const 0
        i32.const 1048896
        i32.store offset=1048908
        i32.const 0
        i32.const 1048888
        i32.store offset=1048896
        i32.const 0
        i32.const 1048904
        i32.store offset=1048916
        i32.const 0
        i32.const 1048896
        i32.store offset=1048904
        i32.const 0
        i32.const 1048912
        i32.store offset=1048924
        i32.const 0
        i32.const 1048904
        i32.store offset=1048912
        i32.const 0
        i32.const 1048920
        i32.store offset=1048932
        i32.const 0
        i32.const 1048912
        i32.store offset=1048920
        i32.const 0
        i32.const 1048928
        i32.store offset=1048940
        i32.const 0
        i32.const 1048920
        i32.store offset=1048928
        i32.const 0
        i32.const 1048936
        i32.store offset=1048948
        i32.const 0
        i32.const 1048928
        i32.store offset=1048936
        i32.const 0
        i32.const 1048944
        i32.store offset=1048956
        i32.const 0
        i32.const 1048936
        i32.store offset=1048944
        i32.const 0
        i32.const 1048952
        i32.store offset=1048964
        i32.const 0
        i32.const 1048944
        i32.store offset=1048952
        i32.const 0
        i32.const 1048960
        i32.store offset=1048972
        i32.const 0
        i32.const 1048952
        i32.store offset=1048960
        i32.const 0
        i32.const 1048968
        i32.store offset=1048980
        i32.const 0
        i32.const 1048960
        i32.store offset=1048968
        i32.const 0
        i32.const 1048976
        i32.store offset=1048988
        i32.const 0
        i32.const 1048968
        i32.store offset=1048976
        i32.const 0
        i32.const 1048976
        i32.store offset=1048984
        call 36
        local.tee 3
        i32.const 8
        call 17
        local.set 4
        i32.const 20
        i32.const 8
        call 17
        local.set 7
        i32.const 16
        i32.const 8
        call 17
        local.set 8
        i32.const 0
        local.get 6
        local.get 6
        call 35
        local.tee 0
        i32.const 8
        call 17
        local.get 0
        i32.sub
        local.tee 10
        call 33
        local.tee 0
        i32.store offset=1049012
        i32.const 0
        local.get 3
        local.get 5
        i32.add
        local.get 8
        local.get 4
        local.get 7
        i32.add
        i32.add
        local.get 10
        i32.add
        i32.sub
        local.tee 6
        i32.store offset=1049004
        local.get 0
        local.get 6
        i32.const 1
        i32.or
        i32.store offset=4
        call 36
        local.tee 3
        i32.const 8
        call 17
        local.set 4
        i32.const 20
        i32.const 8
        call 17
        local.set 5
        i32.const 16
        i32.const 8
        call 17
        local.set 7
        local.get 0
        local.get 6
        call 33
        local.get 7
        local.get 5
        local.get 4
        local.get 3
        i32.sub
        i32.add
        i32.add
        i32.store offset=4
        i32.const 0
        i32.const 2097152
        i32.store offset=1049024
      end
      i32.const 0
      local.set 6
      i32.const 0
      i32.load offset=1049004
      local.tee 0
      local.get 2
      i32.le_u
      br_if 0 (;@1;)
      i32.const 0
      local.get 0
      local.get 2
      i32.sub
      local.tee 6
      i32.store offset=1049004
      i32.const 0
      i32.const 0
      i32.load offset=1049012
      local.tee 0
      local.get 2
      call 33
      local.tee 3
      i32.store offset=1049012
      local.get 3
      local.get 6
      i32.const 1
      i32.or
      i32.store offset=4
      local.get 0
      local.get 2
      call 30
      local.get 0
      call 35
      local.set 6
    end
    local.get 1
    i32.const 16
    i32.add
    global.set 0
    local.get 6
  )
  (func (;9;) (type 4) (param i32 i32)
    (local i32 i32 i32 i32)
    local.get 0
    local.get 1
    call 33
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          local.get 0
          call 24
          br_if 0 (;@3;)
          local.get 0
          i32.load
          local.set 3
          block ;; label = @4
            block ;; label = @5
              local.get 0
              call 27
              br_if 0 (;@5;)
              local.get 3
              local.get 1
              i32.add
              local.set 1
              local.get 0
              local.get 3
              call 34
              local.tee 0
              i32.const 0
              i32.load offset=1049008
              i32.ne
              br_if 1 (;@4;)
              local.get 2
              i32.load offset=4
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 2 (;@3;)
              i32.const 0
              local.get 1
              i32.store offset=1049000
              local.get 0
              local.get 1
              local.get 2
              call 32
              return
            end
            i32.const 1048584
            local.get 0
            local.get 3
            i32.sub
            local.get 3
            local.get 1
            i32.add
            i32.const 16
            i32.add
            local.tee 0
            call 49
            i32.eqz
            br_if 2 (;@2;)
            i32.const 0
            i32.const 0
            i32.load offset=1049016
            local.get 0
            i32.sub
            i32.store offset=1049016
            return
          end
          block ;; label = @4
            local.get 3
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            call 10
            br 1 (;@3;)
          end
          block ;; label = @4
            local.get 0
            i32.const 12
            i32.add
            i32.load
            local.tee 4
            local.get 0
            i32.const 8
            i32.add
            i32.load
            local.tee 5
            i32.eq
            br_if 0 (;@4;)
            local.get 5
            local.get 4
            i32.store offset=12
            local.get 4
            local.get 5
            i32.store offset=8
            br 1 (;@3;)
          end
          i32.const 0
          i32.const 0
          i32.load offset=1048992
          i32.const -2
          local.get 3
          i32.const 3
          i32.shr_u
          i32.rotl
          i32.and
          i32.store offset=1048992
        end
        block ;; label = @3
          local.get 2
          call 23
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 1
          local.get 2
          call 32
          br 2 (;@1;)
        end
        block ;; label = @3
          block ;; label = @4
            local.get 2
            i32.const 0
            i32.load offset=1049012
            i32.eq
            br_if 0 (;@4;)
            local.get 2
            i32.const 0
            i32.load offset=1049008
            i32.ne
            br_if 1 (;@3;)
            i32.const 0
            local.get 0
            i32.store offset=1049008
            i32.const 0
            i32.const 0
            i32.load offset=1049000
            local.get 1
            i32.add
            local.tee 1
            i32.store offset=1049000
            local.get 0
            local.get 1
            call 31
            return
          end
          i32.const 0
          local.get 0
          i32.store offset=1049012
          i32.const 0
          i32.const 0
          i32.load offset=1049004
          local.get 1
          i32.add
          local.tee 1
          i32.store offset=1049004
          local.get 0
          local.get 1
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 0
          i32.const 0
          i32.load offset=1049008
          i32.ne
          br_if 1 (;@2;)
          i32.const 0
          i32.const 0
          i32.store offset=1049000
          i32.const 0
          i32.const 0
          i32.store offset=1049008
          return
        end
        local.get 2
        call 22
        local.tee 3
        local.get 1
        i32.add
        local.set 1
        block ;; label = @3
          block ;; label = @4
            local.get 3
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 2
            call 10
            br 1 (;@3;)
          end
          block ;; label = @4
            local.get 2
            i32.const 12
            i32.add
            i32.load
            local.tee 4
            local.get 2
            i32.const 8
            i32.add
            i32.load
            local.tee 2
            i32.eq
            br_if 0 (;@4;)
            local.get 2
            local.get 4
            i32.store offset=12
            local.get 4
            local.get 2
            i32.store offset=8
            br 1 (;@3;)
          end
          i32.const 0
          i32.const 0
          i32.load offset=1048992
          i32.const -2
          local.get 3
          i32.const 3
          i32.shr_u
          i32.rotl
          i32.and
          i32.store offset=1048992
        end
        local.get 0
        local.get 1
        call 31
        local.get 0
        i32.const 0
        i32.load offset=1049008
        i32.ne
        br_if 1 (;@1;)
        i32.const 0
        local.get 1
        i32.store offset=1049000
      end
      return
    end
    block ;; label = @1
      local.get 1
      i32.const 256
      i32.lt_u
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      call 11
      return
    end
    local.get 1
    i32.const -8
    i32.and
    i32.const 1048728
    i32.add
    local.set 2
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048992
        local.tee 3
        i32.const 1
        local.get 1
        i32.const 3
        i32.shr_u
        i32.shl
        local.tee 1
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.load offset=8
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.get 3
      local.get 1
      i32.or
      i32.store offset=1048992
      local.get 2
      local.set 1
    end
    local.get 2
    local.get 0
    i32.store offset=8
    local.get 1
    local.get 0
    i32.store offset=12
    local.get 0
    local.get 2
    i32.store offset=12
    local.get 0
    local.get 1
    i32.store offset=8
  )
  (func (;10;) (type 5) (param i32)
    (local i32 i32 i32 i32 i32)
    local.get 0
    i32.load offset=24
    local.set 1
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          local.get 0
          call 40
          local.get 0
          i32.ne
          br_if 0 (;@3;)
          local.get 0
          i32.const 20
          i32.const 16
          local.get 0
          i32.const 20
          i32.add
          local.tee 2
          i32.load
          local.tee 3
          select
          i32.add
          i32.load
          local.tee 4
          br_if 1 (;@2;)
          i32.const 0
          local.set 2
          br 2 (;@1;)
        end
        local.get 0
        call 41
        local.tee 4
        local.get 0
        call 40
        local.tee 2
        call 39
        i32.store offset=12
        local.get 2
        local.get 4
        call 39
        i32.store offset=8
        br 1 (;@1;)
      end
      local.get 2
      local.get 0
      i32.const 16
      i32.add
      local.get 3
      select
      local.set 3
      loop ;; label = @2
        local.get 3
        local.set 5
        local.get 4
        local.tee 2
        i32.const 20
        i32.add
        local.tee 4
        local.get 2
        i32.const 16
        i32.add
        local.get 4
        i32.load
        local.tee 4
        select
        local.set 3
        local.get 2
        i32.const 20
        i32.const 16
        local.get 4
        select
        i32.add
        i32.load
        local.tee 4
        br_if 0 (;@2;)
      end
      local.get 5
      i32.const 0
      i32.store
    end
    block ;; label = @1
      local.get 1
      i32.eqz
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          local.get 0
          i32.load offset=28
          i32.const 2
          i32.shl
          i32.const 1048584
          i32.add
          local.tee 4
          i32.load
          local.get 0
          i32.eq
          br_if 0 (;@3;)
          local.get 1
          i32.const 16
          i32.const 20
          local.get 1
          i32.load offset=16
          local.get 0
          i32.eq
          select
          i32.add
          local.get 2
          i32.store
          local.get 2
          i32.eqz
          br_if 2 (;@1;)
          br 1 (;@2;)
        end
        local.get 4
        local.get 2
        i32.store
        local.get 2
        br_if 0 (;@2;)
        i32.const 0
        i32.const 0
        i32.load offset=1048996
        i32.const -2
        local.get 0
        i32.load offset=28
        i32.rotl
        i32.and
        i32.store offset=1048996
        return
      end
      local.get 2
      local.get 1
      i32.store offset=24
      block ;; label = @2
        local.get 0
        i32.load offset=16
        local.tee 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 4
        i32.store offset=16
        local.get 4
        local.get 2
        i32.store offset=24
      end
      local.get 0
      i32.const 20
      i32.add
      i32.load
      local.tee 4
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      i32.const 20
      i32.add
      local.get 4
      i32.store
      local.get 4
      local.get 2
      i32.store offset=24
      return
    end
  )
  (func (;11;) (type 4) (param i32 i32)
    (local i32 i32 i32 i32 i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      local.get 1
      i32.const 256
      i32.lt_u
      br_if 0 (;@1;)
      i32.const 31
      local.set 2
      local.get 1
      i32.const 16777215
      i32.gt_u
      br_if 0 (;@1;)
      local.get 1
      i32.const 6
      local.get 1
      i32.const 8
      i32.shr_u
      i32.clz
      local.tee 2
      i32.sub
      i32.shr_u
      i32.const 1
      i32.and
      local.get 2
      i32.const 1
      i32.shl
      i32.sub
      i32.const 62
      i32.add
      local.set 2
    end
    local.get 0
    i64.const 0
    i64.store offset=16 align=4
    local.get 0
    local.get 2
    i32.store offset=28
    local.get 2
    i32.const 2
    i32.shl
    i32.const 1048584
    i32.add
    local.set 3
    local.get 0
    call 39
    local.set 4
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              i32.const 0
              i32.load offset=1048996
              local.tee 5
              i32.const 1
              local.get 2
              i32.shl
              local.tee 6
              i32.and
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              i32.load
              local.set 5
              local.get 2
              call 20
              local.set 2
              local.get 5
              call 39
              call 22
              local.get 1
              i32.ne
              br_if 1 (;@4;)
              local.get 5
              local.set 2
              br 2 (;@3;)
            end
            i32.const 0
            local.get 5
            local.get 6
            i32.or
            i32.store offset=1048996
            local.get 3
            local.get 0
            i32.store
            local.get 0
            local.get 3
            i32.store offset=24
            br 3 (;@1;)
          end
          local.get 1
          local.get 2
          i32.shl
          local.set 3
          loop ;; label = @4
            local.get 5
            local.get 3
            i32.const 29
            i32.shr_u
            i32.const 4
            i32.and
            i32.add
            i32.const 16
            i32.add
            local.tee 6
            i32.load
            local.tee 2
            i32.eqz
            br_if 2 (;@2;)
            local.get 3
            i32.const 1
            i32.shl
            local.set 3
            local.get 2
            local.set 5
            local.get 2
            call 39
            call 22
            local.get 1
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 2
        call 39
        local.tee 2
        i32.load offset=8
        local.tee 3
        local.get 4
        i32.store offset=12
        local.get 2
        local.get 4
        i32.store offset=8
        local.get 4
        local.get 2
        i32.store offset=12
        local.get 4
        local.get 3
        i32.store offset=8
        local.get 0
        i32.const 0
        i32.store offset=24
        return
      end
      local.get 6
      local.get 0
      i32.store
      local.get 0
      local.get 5
      i32.store offset=24
    end
    local.get 4
    local.get 4
    i32.store offset=8
    local.get 4
    local.get 4
    i32.store offset=12
  )
  (func (;12;) (type 6) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    i32.const 0
    local.set 0
    i32.const 0
    local.set 1
    block ;; label = @1
      i32.const 0
      i32.load offset=1048720
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1048712
      local.set 3
      i32.const 0
      local.set 1
      i32.const 0
      local.set 0
      loop ;; label = @2
        local.get 2
        local.tee 4
        i32.load offset=8
        local.set 2
        local.get 4
        i32.load offset=4
        local.set 5
        local.get 4
        i32.load
        local.set 6
        block ;; label = @3
          block ;; label = @4
            i32.const 1048584
            local.get 4
            i32.const 12
            i32.add
            i32.load
            i32.const 1
            i32.shr_u
            call 50
            i32.eqz
            br_if 0 (;@4;)
            local.get 4
            call 42
            br_if 0 (;@4;)
            local.get 6
            local.get 6
            call 35
            local.tee 7
            i32.const 8
            call 17
            local.get 7
            i32.sub
            i32.add
            local.tee 7
            call 22
            local.set 8
            call 36
            local.tee 9
            i32.const 8
            call 17
            local.set 10
            i32.const 20
            i32.const 8
            call 17
            local.set 11
            i32.const 16
            i32.const 8
            call 17
            local.set 12
            local.get 7
            call 26
            br_if 0 (;@4;)
            local.get 7
            local.get 8
            i32.add
            local.get 6
            local.get 9
            local.get 5
            i32.add
            local.get 10
            local.get 11
            i32.add
            local.get 12
            i32.add
            i32.sub
            i32.add
            i32.lt_u
            br_if 0 (;@4;)
            block ;; label = @5
              block ;; label = @6
                local.get 7
                i32.const 0
                i32.load offset=1049008
                i32.eq
                br_if 0 (;@6;)
                local.get 7
                call 10
                br 1 (;@5;)
              end
              i32.const 0
              i32.const 0
              i32.store offset=1049000
              i32.const 0
              i32.const 0
              i32.store offset=1049008
            end
            block ;; label = @5
              i32.const 1048584
              local.get 6
              local.get 5
              call 49
              br_if 0 (;@5;)
              local.get 7
              local.get 8
              call 11
              br 1 (;@4;)
            end
            i32.const 0
            i32.const 0
            i32.load offset=1049016
            local.get 5
            i32.sub
            i32.store offset=1049016
            local.get 3
            local.get 2
            i32.store offset=8
            local.get 5
            local.get 1
            i32.add
            local.set 1
            br 1 (;@3;)
          end
          local.get 4
          local.set 3
        end
        local.get 0
        i32.const 1
        i32.add
        local.set 0
        local.get 2
        br_if 0 (;@2;)
      end
    end
    i32.const 0
    local.get 0
    i32.const 4095
    local.get 0
    i32.const 4095
    i32.gt_u
    select
    i32.store offset=1049032
    local.get 1
  )
  (func (;13;) (type 5) (param i32)
    (local i32 i32 i32 i32 i32 i32)
    local.get 0
    call 37
    local.set 0
    local.get 0
    local.get 0
    call 22
    local.tee 1
    call 33
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          local.get 0
          call 24
          br_if 0 (;@3;)
          local.get 0
          i32.load
          local.set 3
          block ;; label = @4
            block ;; label = @5
              local.get 0
              call 27
              br_if 0 (;@5;)
              local.get 3
              local.get 1
              i32.add
              local.set 1
              local.get 0
              local.get 3
              call 34
              local.tee 0
              i32.const 0
              i32.load offset=1049008
              i32.ne
              br_if 1 (;@4;)
              local.get 2
              i32.load offset=4
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 2 (;@3;)
              i32.const 0
              local.get 1
              i32.store offset=1049000
              local.get 0
              local.get 1
              local.get 2
              call 32
              return
            end
            i32.const 1048584
            local.get 0
            local.get 3
            i32.sub
            local.get 3
            local.get 1
            i32.add
            i32.const 16
            i32.add
            local.tee 0
            call 49
            i32.eqz
            br_if 2 (;@2;)
            i32.const 0
            i32.const 0
            i32.load offset=1049016
            local.get 0
            i32.sub
            i32.store offset=1049016
            return
          end
          block ;; label = @4
            local.get 3
            i32.const 256
            i32.lt_u
            br_if 0 (;@4;)
            local.get 0
            call 10
            br 1 (;@3;)
          end
          block ;; label = @4
            local.get 0
            i32.const 12
            i32.add
            i32.load
            local.tee 4
            local.get 0
            i32.const 8
            i32.add
            i32.load
            local.tee 5
            i32.eq
            br_if 0 (;@4;)
            local.get 5
            local.get 4
            i32.store offset=12
            local.get 4
            local.get 5
            i32.store offset=8
            br 1 (;@3;)
          end
          i32.const 0
          i32.const 0
          i32.load offset=1048992
          i32.const -2
          local.get 3
          i32.const 3
          i32.shr_u
          i32.rotl
          i32.and
          i32.store offset=1048992
        end
        block ;; label = @3
          block ;; label = @4
            local.get 2
            call 23
            i32.eqz
            br_if 0 (;@4;)
            local.get 0
            local.get 1
            local.get 2
            call 32
            br 1 (;@3;)
          end
          block ;; label = @4
            block ;; label = @5
              block ;; label = @6
                block ;; label = @7
                  local.get 2
                  i32.const 0
                  i32.load offset=1049012
                  i32.eq
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 0
                  i32.load offset=1049008
                  i32.ne
                  br_if 1 (;@6;)
                  i32.const 0
                  local.get 0
                  i32.store offset=1049008
                  i32.const 0
                  i32.const 0
                  i32.load offset=1049000
                  local.get 1
                  i32.add
                  local.tee 1
                  i32.store offset=1049000
                  local.get 0
                  local.get 1
                  call 31
                  return
                end
                i32.const 0
                local.get 0
                i32.store offset=1049012
                i32.const 0
                i32.const 0
                i32.load offset=1049004
                local.get 1
                i32.add
                local.tee 1
                i32.store offset=1049004
                local.get 0
                local.get 1
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                i32.const 0
                i32.load offset=1049008
                i32.eq
                br_if 1 (;@5;)
                br 2 (;@4;)
              end
              local.get 2
              call 22
              local.tee 3
              local.get 1
              i32.add
              local.set 1
              block ;; label = @6
                block ;; label = @7
                  local.get 3
                  i32.const 256
                  i32.lt_u
                  br_if 0 (;@7;)
                  local.get 2
                  call 10
                  br 1 (;@6;)
                end
                block ;; label = @7
                  local.get 2
                  i32.const 12
                  i32.add
                  i32.load
                  local.tee 4
                  local.get 2
                  i32.const 8
                  i32.add
                  i32.load
                  local.tee 2
                  i32.eq
                  br_if 0 (;@7;)
                  local.get 2
                  local.get 4
                  i32.store offset=12
                  local.get 4
                  local.get 2
                  i32.store offset=8
                  br 1 (;@6;)
                end
                i32.const 0
                i32.const 0
                i32.load offset=1048992
                i32.const -2
                local.get 3
                i32.const 3
                i32.shr_u
                i32.rotl
                i32.and
                i32.store offset=1048992
              end
              local.get 0
              local.get 1
              call 31
              local.get 0
              i32.const 0
              i32.load offset=1049008
              i32.ne
              br_if 2 (;@3;)
              i32.const 0
              local.get 1
              i32.store offset=1049000
              br 3 (;@2;)
            end
            i32.const 0
            i32.const 0
            i32.store offset=1049000
            i32.const 0
            i32.const 0
            i32.store offset=1049008
          end
          i32.const 0
          i32.load offset=1049024
          local.get 1
          i32.ge_u
          br_if 1 (;@2;)
          call 36
          local.tee 0
          i32.const 8
          call 17
          local.set 1
          i32.const 20
          i32.const 8
          call 17
          local.set 2
          i32.const 16
          i32.const 8
          call 17
          local.set 3
          i32.const 0
          i32.const 16
          i32.const 8
          call 17
          i32.const 2
          i32.shl
          i32.sub
          local.tee 4
          local.get 0
          local.get 3
          local.get 1
          local.get 2
          i32.add
          i32.add
          i32.sub
          i32.const -65544
          i32.add
          i32.const -9
          i32.and
          i32.const -3
          i32.add
          local.tee 0
          local.get 4
          local.get 0
          i32.lt_u
          select
          i32.eqz
          br_if 1 (;@2;)
          i32.const 0
          i32.load offset=1049012
          i32.eqz
          br_if 1 (;@2;)
          call 36
          local.tee 0
          i32.const 8
          call 17
          local.set 1
          i32.const 20
          i32.const 8
          call 17
          local.set 3
          i32.const 16
          i32.const 8
          call 17
          local.set 4
          i32.const 0
          local.set 2
          block ;; label = @4
            i32.const 0
            i32.load offset=1049004
            local.tee 5
            local.get 4
            local.get 3
            local.get 1
            local.get 0
            i32.sub
            i32.add
            i32.add
            local.tee 0
            i32.le_u
            br_if 0 (;@4;)
            local.get 5
            local.get 0
            i32.sub
            i32.const 65535
            i32.add
            i32.const -65536
            i32.and
            local.tee 4
            i32.const -65536
            i32.add
            local.set 3
            i32.const 0
            i32.load offset=1049012
            local.set 1
            i32.const 1048712
            local.set 0
            block ;; label = @5
              loop ;; label = @6
                block ;; label = @7
                  local.get 0
                  i32.load
                  local.get 1
                  i32.gt_u
                  br_if 0 (;@7;)
                  local.get 0
                  call 45
                  local.get 1
                  i32.gt_u
                  br_if 2 (;@5;)
                end
                local.get 0
                i32.load offset=8
                local.tee 0
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 0
            end
            i32.const 0
            local.set 2
            local.get 0
            call 42
            br_if 0 (;@4;)
            i32.const 1048584
            local.get 0
            i32.const 12
            i32.add
            i32.load
            i32.const 1
            i32.shr_u
            call 50
            i32.eqz
            br_if 0 (;@4;)
            local.get 0
            i32.load offset=4
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            i32.const 1048712
            local.set 1
            loop ;; label = @5
              local.get 0
              local.get 1
              call 44
              br_if 1 (;@4;)
              local.get 1
              i32.load offset=8
              local.tee 1
              br_if 0 (;@5;)
            end
            i32.const 1048584
            local.get 0
            i32.load
            local.get 0
            i32.load offset=4
            local.tee 1
            local.get 1
            local.get 3
            i32.sub
            call 48
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            i32.eqz
            br_if 0 (;@4;)
            local.get 0
            local.get 0
            i32.load offset=4
            local.get 3
            i32.sub
            i32.store offset=4
            i32.const 0
            i32.const 0
            i32.load offset=1049016
            local.get 3
            i32.sub
            i32.store offset=1049016
            i32.const 0
            i32.load offset=1049004
            local.set 1
            i32.const 0
            i32.load offset=1049012
            local.set 0
            i32.const 0
            local.get 0
            local.get 0
            call 35
            local.tee 2
            i32.const 8
            call 17
            local.get 2
            i32.sub
            local.tee 2
            call 33
            local.tee 0
            i32.store offset=1049012
            i32.const 0
            local.get 1
            local.get 4
            local.get 2
            i32.add
            i32.sub
            i32.const 65536
            i32.add
            local.tee 1
            i32.store offset=1049004
            local.get 0
            local.get 1
            i32.const 1
            i32.or
            i32.store offset=4
            call 36
            local.tee 2
            i32.const 8
            call 17
            local.set 4
            i32.const 20
            i32.const 8
            call 17
            local.set 5
            i32.const 16
            i32.const 8
            call 17
            local.set 6
            local.get 0
            local.get 1
            call 33
            local.get 6
            local.get 5
            local.get 4
            local.get 2
            i32.sub
            i32.add
            i32.add
            i32.store offset=4
            i32.const 0
            i32.const 2097152
            i32.store offset=1049024
            local.get 3
            local.set 2
          end
          call 12
          i32.const 0
          local.get 2
          i32.sub
          i32.ne
          br_if 1 (;@2;)
          i32.const 0
          i32.load offset=1049004
          i32.const 0
          i32.load offset=1049024
          i32.le_u
          br_if 1 (;@2;)
          i32.const 0
          i32.const -1
          i32.store offset=1049024
          return
        end
        local.get 1
        i32.const 256
        i32.lt_u
        br_if 1 (;@1;)
        local.get 0
        local.get 1
        call 11
        i32.const 0
        i32.const 0
        i32.load offset=1049032
        i32.const -1
        i32.add
        local.tee 0
        i32.store offset=1049032
        local.get 0
        br_if 0 (;@2;)
        call 12
        drop
        return
      end
      return
    end
    local.get 1
    i32.const -8
    i32.and
    i32.const 1048728
    i32.add
    local.set 2
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048992
        local.tee 3
        i32.const 1
        local.get 1
        i32.const 3
        i32.shr_u
        i32.shl
        local.tee 1
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.load offset=8
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.get 3
      local.get 1
      i32.or
      i32.store offset=1048992
      local.get 2
      local.set 1
    end
    local.get 2
    local.get 0
    i32.store offset=8
    local.get 1
    local.get 0
    i32.store offset=12
    local.get 0
    local.get 2
    i32.store offset=12
    local.get 0
    local.get 1
    i32.store offset=8
  )
  (func (;14;) (type 4) (param i32 i32)
    (local i32 i32 i32 i32)
    local.get 0
    local.get 0
    call 35
    local.tee 2
    i32.const 8
    call 17
    local.get 2
    i32.sub
    local.tee 2
    call 33
    local.set 0
    i32.const 0
    local.get 1
    local.get 2
    i32.sub
    local.tee 1
    i32.store offset=1049004
    i32.const 0
    local.get 0
    i32.store offset=1049012
    local.get 0
    local.get 1
    i32.const 1
    i32.or
    i32.store offset=4
    call 36
    local.tee 2
    i32.const 8
    call 17
    local.set 3
    i32.const 20
    i32.const 8
    call 17
    local.set 4
    i32.const 16
    i32.const 8
    call 17
    local.set 5
    local.get 0
    local.get 1
    call 33
    local.get 5
    local.get 4
    local.get 3
    local.get 2
    i32.sub
    i32.add
    i32.add
    i32.store offset=4
    i32.const 0
    i32.const 2097152
    i32.store offset=1049024
  )
  (func (;15;) (type 2) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call 7
  )
  (func (;16;) (type 3) (param i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            local.get 2
            i32.const 9
            i32.lt_u
            br_if 0 (;@4;)
            local.get 3
            local.get 2
            call 7
            local.tee 2
            br_if 1 (;@3;)
            i32.const 0
            return
          end
          call 36
          local.tee 1
          i32.const 8
          call 17
          local.set 4
          i32.const 20
          i32.const 8
          call 17
          local.set 5
          i32.const 16
          i32.const 8
          call 17
          local.set 6
          i32.const 0
          local.set 2
          i32.const 0
          i32.const 16
          i32.const 8
          call 17
          i32.const 2
          i32.shl
          i32.sub
          local.tee 7
          local.get 1
          local.get 6
          local.get 4
          local.get 5
          i32.add
          i32.add
          i32.sub
          i32.const -65544
          i32.add
          i32.const -9
          i32.and
          i32.const -3
          i32.add
          local.tee 1
          local.get 7
          local.get 1
          i32.lt_u
          select
          local.get 3
          i32.le_u
          br_if 1 (;@2;)
          i32.const 16
          local.get 3
          i32.const 4
          i32.add
          i32.const 16
          i32.const 8
          call 17
          i32.const -5
          i32.add
          local.get 3
          i32.gt_u
          select
          i32.const 8
          call 17
          local.set 4
          local.get 0
          call 37
          local.set 1
          local.get 1
          local.get 1
          call 22
          local.tee 5
          call 33
          local.set 6
          block ;; label = @4
            block ;; label = @5
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    block ;; label = @9
                      block ;; label = @10
                        block ;; label = @11
                          local.get 1
                          call 27
                          br_if 0 (;@11;)
                          local.get 5
                          local.get 4
                          i32.ge_u
                          br_if 1 (;@10;)
                          local.get 6
                          i32.const 0
                          i32.load offset=1049012
                          i32.eq
                          br_if 2 (;@9;)
                          local.get 6
                          i32.const 0
                          i32.load offset=1049008
                          i32.eq
                          br_if 3 (;@8;)
                          local.get 6
                          call 23
                          br_if 7 (;@4;)
                          local.get 6
                          call 22
                          local.tee 7
                          local.get 5
                          i32.add
                          local.tee 5
                          local.get 4
                          i32.lt_u
                          br_if 7 (;@4;)
                          local.get 5
                          local.get 4
                          i32.sub
                          local.set 8
                          local.get 7
                          i32.const 256
                          i32.lt_u
                          br_if 4 (;@7;)
                          local.get 6
                          call 10
                          br 5 (;@6;)
                        end
                        local.get 1
                        call 22
                        local.set 5
                        local.get 4
                        i32.const 256
                        i32.lt_u
                        br_if 6 (;@4;)
                        block ;; label = @11
                          local.get 5
                          local.get 4
                          i32.const 4
                          i32.add
                          i32.lt_u
                          br_if 0 (;@11;)
                          local.get 5
                          local.get 4
                          i32.sub
                          i32.const 131073
                          i32.lt_u
                          br_if 6 (;@5;)
                        end
                        i32.const 1048584
                        local.get 1
                        local.get 1
                        i32.load
                        local.tee 6
                        i32.sub
                        local.get 5
                        local.get 6
                        i32.add
                        i32.const 16
                        i32.add
                        local.tee 7
                        local.get 4
                        i32.const 31
                        i32.add
                        i32.const 1048584
                        call 51
                        call 17
                        local.tee 5
                        i32.const 1
                        call 47
                        local.tee 4
                        i32.eqz
                        br_if 6 (;@4;)
                        local.get 4
                        local.get 6
                        i32.add
                        local.tee 1
                        local.get 5
                        local.get 6
                        i32.sub
                        local.tee 3
                        i32.const -16
                        i32.add
                        local.tee 2
                        i32.store offset=4
                        call 21
                        local.set 0
                        local.get 1
                        local.get 2
                        call 33
                        local.get 0
                        i32.store offset=4
                        local.get 1
                        local.get 3
                        i32.const -12
                        i32.add
                        call 33
                        i32.const 0
                        i32.store offset=4
                        i32.const 0
                        i32.const 0
                        i32.load offset=1049016
                        local.get 5
                        local.get 7
                        i32.sub
                        i32.add
                        local.tee 3
                        i32.store offset=1049016
                        i32.const 0
                        i32.const 0
                        i32.load offset=1049028
                        local.tee 2
                        local.get 4
                        local.get 4
                        local.get 2
                        i32.gt_u
                        select
                        i32.store offset=1049028
                        i32.const 0
                        i32.const 0
                        i32.load offset=1049020
                        local.tee 2
                        local.get 3
                        local.get 2
                        local.get 3
                        i32.gt_u
                        select
                        i32.store offset=1049020
                        br 9 (;@1;)
                      end
                      local.get 5
                      local.get 4
                      i32.sub
                      local.tee 5
                      i32.const 16
                      i32.const 8
                      call 17
                      i32.lt_u
                      br_if 4 (;@5;)
                      local.get 1
                      local.get 4
                      call 33
                      local.set 6
                      local.get 1
                      local.get 4
                      call 28
                      local.get 6
                      local.get 5
                      call 28
                      local.get 6
                      local.get 5
                      call 9
                      br 4 (;@5;)
                    end
                    i32.const 0
                    i32.load offset=1049004
                    local.get 5
                    i32.add
                    local.tee 5
                    local.get 4
                    i32.le_u
                    br_if 4 (;@4;)
                    local.get 1
                    local.get 4
                    call 33
                    local.set 6
                    local.get 1
                    local.get 4
                    call 28
                    local.get 6
                    local.get 5
                    local.get 4
                    i32.sub
                    local.tee 4
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    i32.const 0
                    local.get 4
                    i32.store offset=1049004
                    i32.const 0
                    local.get 6
                    i32.store offset=1049012
                    br 3 (;@5;)
                  end
                  i32.const 0
                  i32.load offset=1049000
                  local.get 5
                  i32.add
                  local.tee 5
                  local.get 4
                  i32.lt_u
                  br_if 3 (;@4;)
                  block ;; label = @8
                    block ;; label = @9
                      local.get 5
                      local.get 4
                      i32.sub
                      local.tee 6
                      i32.const 16
                      i32.const 8
                      call 17
                      i32.ge_u
                      br_if 0 (;@9;)
                      local.get 1
                      local.get 5
                      call 28
                      i32.const 0
                      local.set 6
                      i32.const 0
                      local.set 5
                      br 1 (;@8;)
                    end
                    local.get 1
                    local.get 4
                    call 33
                    local.tee 5
                    local.get 6
                    call 33
                    local.set 7
                    local.get 1
                    local.get 4
                    call 28
                    local.get 5
                    local.get 6
                    call 31
                    local.get 7
                    call 25
                  end
                  i32.const 0
                  local.get 5
                  i32.store offset=1049008
                  i32.const 0
                  local.get 6
                  i32.store offset=1049000
                  br 2 (;@5;)
                end
                block ;; label = @7
                  local.get 6
                  i32.const 12
                  i32.add
                  i32.load
                  local.tee 9
                  local.get 6
                  i32.const 8
                  i32.add
                  i32.load
                  local.tee 6
                  i32.eq
                  br_if 0 (;@7;)
                  local.get 6
                  local.get 9
                  i32.store offset=12
                  local.get 9
                  local.get 6
                  i32.store offset=8
                  br 1 (;@6;)
                end
                i32.const 0
                i32.const 0
                i32.load offset=1048992
                i32.const -2
                local.get 7
                i32.const 3
                i32.shr_u
                i32.rotl
                i32.and
                i32.store offset=1048992
              end
              block ;; label = @6
                local.get 8
                i32.const 16
                i32.const 8
                call 17
                i32.lt_u
                br_if 0 (;@6;)
                local.get 1
                local.get 4
                call 33
                local.set 5
                local.get 1
                local.get 4
                call 28
                local.get 5
                local.get 8
                call 28
                local.get 5
                local.get 8
                call 9
                br 1 (;@5;)
              end
              local.get 1
              local.get 5
              call 28
            end
            local.get 1
            br_if 3 (;@1;)
          end
          local.get 3
          call 8
          local.tee 4
          i32.eqz
          br_if 1 (;@2;)
          local.get 4
          local.get 0
          local.get 1
          call 22
          i32.const -8
          i32.const -4
          local.get 1
          call 27
          select
          i32.add
          local.tee 2
          local.get 3
          local.get 2
          local.get 3
          i32.lt_u
          select
          call 52
          local.set 3
          local.get 0
          call 13
          local.get 3
          return
        end
        local.get 2
        local.get 0
        local.get 1
        local.get 3
        local.get 1
        local.get 3
        i32.lt_u
        select
        call 52
        drop
        local.get 0
        call 13
      end
      local.get 2
      return
    end
    local.get 1
    call 27
    drop
    local.get 1
    call 35
  )
  (func (;17;) (type 2) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    i32.const -1
    i32.add
    i32.const 0
    local.get 1
    i32.sub
    i32.and
  )
  (func (;18;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.shl
    local.tee 0
    i32.const 0
    local.get 0
    i32.sub
    i32.or
  )
  (func (;19;) (type 1) (param i32) (result i32)
    i32.const 0
    local.get 0
    i32.sub
    local.get 0
    i32.and
  )
  (func (;20;) (type 1) (param i32) (result i32)
    i32.const 0
    i32.const 25
    local.get 0
    i32.const 1
    i32.shr_u
    i32.sub
    local.get 0
    i32.const 31
    i32.eq
    select
  )
  (func (;21;) (type 6) (result i32)
    i32.const 7
  )
  (func (;22;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=4
    i32.const -8
    i32.and
  )
  (func (;23;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load8_u offset=4
    i32.const 2
    i32.and
    i32.const 1
    i32.shr_u
  )
  (func (;24;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=4
    i32.const 1
    i32.and
  )
  (func (;25;) (type 5) (param i32)
    local.get 0
    local.get 0
    i32.load offset=4
    i32.const -2
    i32.and
    i32.store offset=4
  )
  (func (;26;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=4
    i32.const 3
    i32.and
    i32.const 1
    i32.ne
  )
  (func (;27;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load8_u offset=4
    i32.const 3
    i32.and
    i32.eqz
  )
  (func (;28;) (type 4) (param i32 i32)
    local.get 0
    local.get 0
    i32.load offset=4
    i32.const 1
    i32.and
    local.get 1
    i32.or
    i32.const 2
    i32.or
    i32.store offset=4
    local.get 0
    local.get 1
    i32.add
    local.tee 0
    local.get 0
    i32.load offset=4
    i32.const 1
    i32.or
    i32.store offset=4
  )
  (func (;29;) (type 4) (param i32 i32)
    local.get 0
    local.get 1
    i32.const 3
    i32.or
    i32.store offset=4
    local.get 0
    local.get 1
    i32.add
    local.tee 1
    local.get 1
    i32.load offset=4
    i32.const 1
    i32.or
    i32.store offset=4
  )
  (func (;30;) (type 4) (param i32 i32)
    local.get 0
    local.get 1
    i32.const 3
    i32.or
    i32.store offset=4
  )
  (func (;31;) (type 4) (param i32 i32)
    local.get 0
    local.get 1
    i32.const 1
    i32.or
    i32.store offset=4
    local.get 0
    local.get 1
    i32.add
    local.get 1
    i32.store
  )
  (func (;32;) (type 7) (param i32 i32 i32)
    local.get 2
    local.get 2
    i32.load offset=4
    i32.const -2
    i32.and
    i32.store offset=4
    local.get 0
    local.get 1
    i32.const 1
    i32.or
    i32.store offset=4
    local.get 0
    local.get 1
    i32.add
    local.get 1
    i32.store
  )
  (func (;33;) (type 2) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func (;34;) (type 2) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.sub
  )
  (func (;35;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 8
    i32.add
  )
  (func (;36;) (type 6) (result i32)
    i32.const 8
  )
  (func (;37;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const -8
    i32.add
  )
  (func (;38;) (type 1) (param i32) (result i32)
    (local i32)
    block ;; label = @1
      local.get 0
      i32.load offset=16
      local.tee 1
      br_if 0 (;@1;)
      local.get 0
      i32.const 20
      i32.add
      i32.load
      local.set 1
    end
    local.get 1
  )
  (func (;39;) (type 1) (param i32) (result i32)
    local.get 0
  )
  (func (;40;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=12
  )
  (func (;41;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=8
  )
  (func (;42;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=12
    i32.const 1
    i32.and
  )
  (func (;43;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=12
    i32.const 1
    i32.shr_u
  )
  (func (;44;) (type 2) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      local.get 0
      i32.load
      local.tee 3
      local.get 1
      i32.gt_u
      br_if 0 (;@1;)
      local.get 3
      local.get 0
      i32.load offset=4
      i32.add
      local.get 1
      i32.gt_u
      local.set 2
    end
    local.get 2
  )
  (func (;45;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    i32.add
  )
  (func (;46;) (type 7) (param i32 i32 i32)
    (local i32)
    local.get 2
    i32.const 16
    i32.shr_u
    memory.grow
    local.set 3
    local.get 0
    i32.const 0
    i32.store offset=8
    local.get 0
    i32.const 0
    local.get 2
    i32.const -65536
    i32.and
    local.get 3
    i32.const -1
    i32.eq
    local.tee 2
    select
    i32.store offset=4
    local.get 0
    i32.const 0
    local.get 3
    i32.const 16
    i32.shl
    local.get 2
    select
    i32.store
  )
  (func (;47;) (type 8) (param i32 i32 i32 i32 i32) (result i32)
    i32.const 0
  )
  (func (;48;) (type 3) (param i32 i32 i32 i32) (result i32)
    i32.const 0
  )
  (func (;49;) (type 9) (param i32 i32 i32) (result i32)
    i32.const 0
  )
  (func (;50;) (type 2) (param i32 i32) (result i32)
    i32.const 0
  )
  (func (;51;) (type 1) (param i32) (result i32)
    i32.const 65536
  )
  (func (;52;) (type 9) (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call 53
  )
  (func (;53;) (type 9) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        local.get 2
        i32.const 15
        i32.gt_u
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        br 1 (;@1;)
      end
      local.get 0
      i32.const 0
      local.get 0
      i32.sub
      i32.const 3
      i32.and
      local.tee 4
      i32.add
      local.set 5
      block ;; label = @2
        local.get 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        local.get 1
        local.set 6
        loop ;; label = @3
          local.get 3
          local.get 6
          i32.load8_u
          i32.store8
          local.get 6
          i32.const 1
          i32.add
          local.set 6
          local.get 3
          i32.const 1
          i32.add
          local.tee 3
          local.get 5
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 5
      local.get 2
      local.get 4
      i32.sub
      local.tee 7
      i32.const -4
      i32.and
      local.tee 8
      i32.add
      local.set 3
      block ;; label = @2
        block ;; label = @3
          local.get 1
          local.get 4
          i32.add
          local.tee 9
          i32.const 3
          i32.and
          local.tee 6
          i32.eqz
          br_if 0 (;@3;)
          local.get 8
          i32.const 1
          i32.lt_s
          br_if 1 (;@2;)
          local.get 9
          i32.const -4
          i32.and
          local.tee 10
          i32.const 4
          i32.add
          local.set 1
          i32.const 0
          local.get 6
          i32.const 3
          i32.shl
          local.tee 2
          i32.sub
          i32.const 24
          i32.and
          local.set 4
          local.get 10
          i32.load
          local.set 6
          loop ;; label = @4
            local.get 5
            local.get 6
            local.get 2
            i32.shr_u
            local.get 1
            i32.load
            local.tee 6
            local.get 4
            i32.shl
            i32.or
            i32.store
            local.get 1
            i32.const 4
            i32.add
            local.set 1
            local.get 5
            i32.const 4
            i32.add
            local.tee 5
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            br 2 (;@2;)
          end
        end
        local.get 8
        i32.const 1
        i32.lt_s
        br_if 0 (;@2;)
        local.get 9
        local.set 1
        loop ;; label = @3
          local.get 5
          local.get 1
          i32.load
          i32.store
          local.get 1
          i32.const 4
          i32.add
          local.set 1
          local.get 5
          i32.const 4
          i32.add
          local.tee 5
          local.get 3
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 7
      i32.const 3
      i32.and
      local.set 2
      local.get 9
      local.get 8
      i32.add
      local.set 1
    end
    block ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      local.get 2
      i32.add
      local.set 5
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.load8_u
        i32.store8
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 3
        i32.const 1
        i32.add
        local.tee 3
        local.get 5
        i32.lt_u
        br_if 0 (;@2;)
      end
    end
    local.get 0
  )
  (table (;0;) 2 2 funcref)
  (memory (;0;) 17)
  (global (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1049036)
  (global (;2;) i32 i32.const 1049040)
  (export "memory" (memory 0))
  (export "add-two" (func 2))
  (export "cabi_realloc" (func 6))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func 1)
  (data (;0;) (i32.const 1048576) "\01\00\00\00")
)
